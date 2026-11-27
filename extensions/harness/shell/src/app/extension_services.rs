//! Running the extensions' background services through `harness-tasks`, each
//! only while the instance's enablement admits its extension.
//!
//! A service starts when its extension is admitted, is cancelled when the
//! enablement withdraws it, and starts again when it is admitted again. One
//! that ends on its own is not restarted until the enablement changes, so a
//! service that cannot run does not spin.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::fmt;
use std::sync::Arc;

use harness_tasks::{BackgroundTasks, CancellationToken};
use sp_extension::{
    BackgroundService, Contributed, EnabledExtensions, ExtensionId, SpContext, SpExtensions,
};
use tokio::sync::watch;

type EnablementFeed = watch::Receiver<Option<EnabledExtensions>>;

/// One extension's background service and what decides whether it may run:
/// the resolved extensions, so an extension whose dependency is withheld is
/// withheld too.
pub struct AdmittedService {
    pub extension: ExtensionId,
    pub service: Arc<dyn BackgroundService>,
    pub extensions: SpExtensions,
}

enum Stopped {
    Withdrawn,
    Ended,
    Cancelled,
}

pub(crate) fn spawn_all(tasks: &BackgroundTasks, extensions: &SpExtensions, ctx: &SpContext) {
    for Contributed { extension, item } in extensions.services(ctx) {
        let name = item.name();
        let admitted = AdmittedService {
            extension,
            service: item,
            extensions: extensions.clone(),
        };
        let enablement = ctx.enablement.subscribe();
        tasks.spawn_cancellable(name, move |cancel| admitted.run(enablement, cancel));
    }
}

impl AdmittedService {
    pub async fn run(self, mut enablement: EnablementFeed, cancel: CancellationToken) {
        let service = self.service.name();
        loop {
            if !self.wait_for_admission(&mut enablement, &cancel).await {
                return;
            }
            tracing::info!(extension = %self.extension, service, "extension service started");
            match self.run_admitted(&mut enablement, &cancel).await {
                Stopped::Cancelled => return,
                Stopped::Withdrawn => {
                    tracing::info!(extension = %self.extension, service, "extension service stopped; the extension is withheld");
                },
                Stopped::Ended => {
                    tracing::warn!(extension = %self.extension, service, "extension service ended on its own");
                    if !next_change(&mut enablement, &cancel).await {
                        return;
                    }
                },
            }
        }
    }

    fn admits(&self, enabled: Option<&EnabledExtensions>) -> bool {
        self.extensions
            .admitted(enabled)
            .iter()
            .any(|entry| entry.metadata.id == self.extension)
    }

    async fn wait_for_admission(
        &self,
        enablement: &mut EnablementFeed,
        cancel: &CancellationToken,
    ) -> bool {
        loop {
            if self.admits(enablement.borrow_and_update().as_ref()) {
                return true;
            }
            if !next_change(enablement, cancel).await {
                return false;
            }
        }
    }

    async fn run_admitted(
        &self,
        enablement: &mut EnablementFeed,
        cancel: &CancellationToken,
    ) -> Stopped {
        let stop = cancel.child_token();
        let mut run = self.service.run(stop.clone());
        loop {
            tokio::select! {
                () = &mut run => return stopped(cancel, Stopped::Ended),
                changed = enablement.changed() => {
                    let admitted =
                        changed.is_ok() && self.admits(enablement.borrow_and_update().as_ref());
                    if !admitted {
                        stop.cancel();
                        run.await;
                        return stopped(cancel, Stopped::Withdrawn);
                    }
                },
            }
        }
    }
}

fn stopped(cancel: &CancellationToken, otherwise: Stopped) -> Stopped {
    if cancel.is_cancelled() {
        Stopped::Cancelled
    } else {
        otherwise
    }
}

async fn next_change(enablement: &mut EnablementFeed, cancel: &CancellationToken) -> bool {
    tokio::select! {
        () = cancel.cancelled() => false,
        changed = enablement.changed() => changed.is_ok(),
    }
}

impl fmt::Debug for AdmittedService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdmittedService")
            .field("extension", &self.extension)
            .field("service", &self.service.name())
            .finish_non_exhaustive()
    }
}
