//! The bridge between the tokio runtime, where the Drive client and the sync
//! call the instance and the disk, and the UI thread.
//!
//! `run` hands a future to an [`OwnedTask`] on the runtime and `blocking` hands
//! it synchronous work; each applies the output to the view, and dropping the
//! returned `Task` (the view closed) drops the owned task, which aborts the
//! work. `follow` keeps a view current from a watch channel: tokio's receivers
//! need no tokio context to be awaited, so the waiting side runs on the UI
//! thread's executor and ends with the view.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::future::Future;

use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use harness_tasks::{OwnedTask, TaskError};
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
    apply(cx, name, job, done)
}

pub(crate) fn blocking<T, R, Work, Done>(
    cx: &Context<T>,
    runtime: &Handle,
    name: &'static str,
    work: Work,
    done: Done,
) -> Task<()>
where
    T: 'static,
    R: Send + 'static,
    Work: FnOnce() -> R + Send + 'static,
    Done: FnOnce(&mut T, R, &mut Context<T>) + 'static,
{
    let job = {
        let _entered = runtime.enter();
        OwnedTask::spawn_blocking(name, work)
    };
    apply(cx, name, job, done)
}

fn apply<T, R, Done>(cx: &Context<T>, name: &'static str, job: OwnedTask<R>, done: Done) -> Task<()>
where
    T: 'static,
    R: Send + 'static,
    Done: FnOnce(&mut T, R, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        let joined: Result<R, TaskError> = job.join().await;
        match joined {
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
        }
    })
}

pub(crate) fn follow<T, V, Apply>(
    cx: &Context<T>,
    mut feed: watch::Receiver<V>,
    apply: Apply,
) -> Task<()>
where
    T: 'static,
    V: Send + Sync + 'static,
    Apply: Fn(&mut T, &mut Context<T>) + 'static,
{
    cx.spawn(async move |this: WeakEntity<T>, cx: &mut AsyncApp| {
        while feed.changed().await.is_ok() {
            if this.update(cx, |this, cx| apply(this, cx)).is_err() {
                break;
            }
        }
    })
}
