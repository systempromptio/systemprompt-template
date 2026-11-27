//! The bridge between the tokio runtime, where the engine, the store and the
//! accounts live, and GPUI's UI thread.
//!
//! Every engine or store call a view or state entity makes goes through these
//! helpers, so the UI thread only ever applies results:
//!
//! - `spawn` runs a future on tokio and hands its output to an entity on the UI
//!   thread.
//! - `blocking` does the same for synchronous work (store reads and writes) on
//!   tokio's blocking pool.
//! - `pump` and `pump_batched` keep an entity current from a broadcast or watch
//!   receiver, reporting lag so the entity can reload what it shows.
//!   `pump_batched` drains everything queued into one update and waits a frame
//!   before the next, so a burst of streaming deltas costs one notify per
//!   frame.
//!
//! Tokio's channel receivers do not need a tokio context to be awaited, so the
//! receiving side runs on GPUI's foreground executor directly. A result that
//! arrives after its entity was released is dropped.

use std::future::Future;
use std::time::Duration;

use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use tokio::runtime::Handle;
use tokio::sync::{broadcast, watch};

pub(crate) const FRAME: Duration = Duration::from_millis(16);

pub(crate) fn spawn<T, R, Fut, Done>(
    cx: &Context<T>,
    runtime: &Handle,
    future: Fut,
    done: Done,
) -> Task<()>
where
    T: 'static,
    R: Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    Done: FnOnce(&mut T, R, &mut Context<T>) + 'static,
{
    let job = runtime.spawn(future);
    cx.spawn(
        async move |this: WeakEntity<T>, cx: &mut AsyncApp| match job.await {
            Ok(value) => {
                if this
                    .update(cx, move |this, cx| done(this, value, cx))
                    .is_err()
                {
                    tracing::trace!("result arrived after its view closed");
                }
            },
            Err(error) => tracing::error!(error = %error, "background task failed"),
        },
    )
}

pub(crate) fn blocking<T, R, Work, Done>(
    cx: &Context<T>,
    runtime: &Handle,
    work: Work,
    done: Done,
) -> Task<()>
where
    T: 'static,
    R: Send + 'static,
    Work: FnOnce() -> R + Send + 'static,
    Done: FnOnce(&mut T, R, &mut Context<T>) + 'static,
{
    let job = runtime.spawn_blocking(work);
    cx.spawn(
        async move |this: WeakEntity<T>, cx: &mut AsyncApp| match job.await {
            Ok(value) => {
                if this
                    .update(cx, move |this, cx| done(this, value, cx))
                    .is_err()
                {
                    tracing::trace!("result arrived after its view closed");
                }
            },
            Err(error) => tracing::error!(error = %error, "blocking task failed"),
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Fed<T> {
    Item(T),
    Lagged(u64),
    Closed,
}

pub(crate) trait Feed: 'static {
    type Item: 'static;

    fn next(&mut self) -> impl Future<Output = Fed<Self::Item>>;

    fn drain(&mut self, _into: &mut Vec<Self::Item>) -> Option<u64> {
        None
    }
}

impl<T: Clone + Send + 'static> Feed for broadcast::Receiver<T> {
    type Item = T;

    async fn next(&mut self) -> Fed<T> {
        match self.recv().await {
            Ok(item) => Fed::Item(item),
            Err(broadcast::error::RecvError::Lagged(missed)) => Fed::Lagged(missed),
            Err(broadcast::error::RecvError::Closed) => Fed::Closed,
        }
    }

    fn drain(&mut self, into: &mut Vec<T>) -> Option<u64> {
        loop {
            match self.try_recv() {
                Ok(item) => into.push(item),
                Err(broadcast::error::TryRecvError::Lagged(missed)) => return Some(missed),
                Err(
                    broadcast::error::TryRecvError::Empty | broadcast::error::TryRecvError::Closed,
                ) => return None,
            }
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Feed for watch::Receiver<T> {
    type Item = T;

    async fn next(&mut self) -> Fed<T> {
        match self.changed().await {
            Ok(()) => Fed::Item(self.borrow_and_update().clone()),
            Err(_) => Fed::Closed,
        }
    }
}

pub(crate) fn pump<T, F, OnItem, OnLag>(
    cx: &Context<T>,
    mut feed: F,
    mut on_item: OnItem,
    mut on_lag: OnLag,
) -> Task<()>
where
    T: 'static,
    F: Feed,
    OnItem: FnMut(&mut T, F::Item, &mut Context<T>) + 'static,
    OnLag: FnMut(&mut T, u64, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        loop {
            let alive = match feed.next().await {
                Fed::Item(item) => this.update(cx, |this, cx| on_item(this, item, cx)).is_ok(),
                Fed::Lagged(missed) => {
                    tracing::debug!(missed, "feed lagged");
                    this.update(cx, |this, cx| on_lag(this, missed, cx)).is_ok()
                },
                Fed::Closed => false,
            };
            if !alive {
                break;
            }
        }
    })
}

pub(crate) fn pump_batched<T, F, OnBatch, OnLag>(
    cx: &Context<T>,
    mut feed: F,
    spacing: Duration,
    mut on_batch: OnBatch,
    mut on_lag: OnLag,
) -> Task<()>
where
    T: 'static,
    F: Feed,
    OnBatch: FnMut(&mut T, Vec<F::Item>, &mut Context<T>) + 'static,
    OnLag: FnMut(&mut T, u64, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        loop {
            let mut batch = Vec::new();
            let mut lagged = match feed.next().await {
                Fed::Item(item) => {
                    batch.push(item);
                    None
                },
                Fed::Lagged(missed) => Some(missed),
                Fed::Closed => break,
            };
            if let Some(missed) = feed.drain(&mut batch) {
                lagged = Some(lagged.unwrap_or(0).saturating_add(missed));
            }
            let alive = this
                .update(cx, |this, cx| {
                    if let Some(missed) = lagged {
                        tracing::debug!(missed, "feed lagged");
                        on_lag(this, missed, cx);
                    }
                    if !batch.is_empty() {
                        on_batch(this, batch, cx);
                    }
                })
                .is_ok();
            if !alive {
                break;
            }
            cx.background_executor().timer(spacing).await;
        }
    })
}
