//! The bridge between the tokio runtime the brain's calls run on and GPUI's UI
//! thread.
//!
//! `run` starts a future as an [`OwnedTask`] on the runtime and applies its
//! output to the entity on the UI thread; the returned GPUI task owns the owned
//! task, so dropping it (a view closing, a read superseded) aborts the call.
//! `follow` keeps an entity current from a watch receiver; tokio's receivers
//! need no runtime context to be awaited, so it runs on GPUI's foreground
//! executor directly.

use std::future::Future;

use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use systemprompt_traits::OwnedTask;
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
        async move |this: WeakEntity<T>, cx: &mut AsyncApp| match job.join_named().await {
            Ok(value) => {
                if this
                    .update(cx, move |this, cx| done(this, value, cx))
                    .is_err()
                {
                    tracing::trace!(task = name, "brain result arrived after its view closed");
                }
            },
            Err(error) if error.is_cancelled() => {
                tracing::trace!(task = name, "brain task cancelled");
            },
            Err(error) => tracing::error!(task = name, error = %error, "brain task failed"),
        },
    )
}

pub(crate) fn follow<T, V, OnChange>(
    cx: &Context<T>,
    mut receiver: watch::Receiver<V>,
    mut on_change: OnChange,
) -> Task<()>
where
    T: 'static,
    V: Clone + Send + Sync + 'static,
    OnChange: FnMut(&mut T, V, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        while receiver.changed().await.is_ok() {
            let value = receiver.borrow_and_update().clone();
            if this
                .update(cx, |this, cx| on_change(this, value, cx))
                .is_err()
            {
                break;
            }
        }
    })
}
