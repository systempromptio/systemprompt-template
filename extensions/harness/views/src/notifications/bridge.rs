//! The bridge between the tokio runtime and the UI thread.
//!
//! The sp handles call the instance on tokio; views apply results on the UI
//! thread.
//!
//! `run` hands a future to an [`OwnedTask`] on the runtime and applies its
//! output to the view; dropping the returned `Task` (the view closed) drops the
//! owned task, which aborts the call. `follow` keeps a view current from a
//! change feed: tokio's receivers need no tokio context to be awaited, so the
//! waiting side runs on the UI thread's executor and ends with the view.

use std::future::Future;

use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use harness_tasks::OwnedTask;
use sp_slack::SlackChanges;
use tokio::runtime::Handle;
use tokio::sync::watch;

pub(crate) fn run<T, R, Fut, Done>(
    cx: &Context<T>,
    runtime: &Handle,
    name: &'static str,
    future: Fut,
    done: Done,
) -> Task<()>
where
    T: 'static,
    R: Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    Done: FnOnce(&mut T, R, &mut Context<T>) + 'static,
{
    let job = {
        let _entered = runtime.enter();
        OwnedTask::spawn(name, future)
    };
    cx.spawn(
        async move |this: WeakEntity<T>, cx: &mut AsyncApp| match job.join().await {
            Ok(value) => {
                if this
                    .update(cx, move |this, cx| done(this, value, cx))
                    .is_err()
                {
                    tracing::trace!(task = name, "result arrived after its view closed");
                }
            },
            Err(error) => {
                tracing::warn!(task = name, error = %error, "view task ended without a result");
            },
        },
    )
}

pub(crate) trait Changes: 'static {
    fn next_change(&mut self) -> impl Future<Output = bool>;
}

impl<T: Send + Sync + 'static> Changes for watch::Receiver<T> {
    async fn next_change(&mut self) -> bool {
        self.changed().await.is_ok()
    }
}

impl Changes for SlackChanges {
    async fn next_change(&mut self) -> bool {
        self.changed().await
    }
}

pub(crate) fn follow<T, Feed, Apply>(cx: &Context<T>, mut feed: Feed, apply: Apply) -> Task<()>
where
    T: 'static,
    Feed: Changes,
    Apply: Fn(&mut T, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        while feed.next_change().await {
            if this.update(cx, |this, cx| apply(this, cx)).is_err() {
                break;
            }
        }
    })
}
