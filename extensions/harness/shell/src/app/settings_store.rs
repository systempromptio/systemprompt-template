//! The owner of the shell's preferences, and the preference store extensions
//! reach through `SettingsAccess`.
//!
//! `SettingsStore` is the single owner. Every change goes through
//! `SettingsStore::update`, which bumps the revision extensions follow; a
//! background task writes the latest value once changes have been quiet for a
//! moment, atomically and owner-only. A file that does not decode is logged and
//! replaced by defaults on the next write: these are preferences, not data.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use harness_ids::ExtensionId;
use harness_tasks::{BackgroundTasks, CancellationToken, OwnedTask};
use serde_json::{Map, Value};
use sp_extension::SettingsAccess;
use sp_secure::SecureError;
use tokio::sync::watch;

use crate::app::settings::UiSettings;

const SAVE_DELAY: Duration = Duration::from_millis(400);
const SAVER_TASK: &str = "preferences-saver";
const WRITER_TASK: &str = "preferences-writer";

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("the preferences could not be encoded")]
    Encode(#[source] serde_json::Error),
    #[error("the preferences file could not be written")]
    Write(#[source] SecureError),
}

#[derive(Debug, Clone)]
pub struct SettingsStore {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    path: PathBuf,
    current: watch::Sender<UiSettings>,
    revision: watch::Sender<u64>,
}

impl SettingsStore {
    pub fn load(path: PathBuf) -> Self {
        let settings = read(&path);
        let (current, _) = watch::channel(settings);
        let (revision, _) = watch::channel(0);
        Self {
            inner: Arc::new(Inner {
                path,
                current,
                revision,
            }),
        }
    }

    pub fn get(&self) -> UiSettings {
        self.inner.current.borrow().clone()
    }

    pub(crate) fn read<R>(&self, read: impl FnOnce(&UiSettings) -> R) -> R {
        read(&self.inner.current.borrow())
    }

    pub fn update(&self, edit: impl FnOnce(&mut UiSettings)) {
        let changed = self.inner.current.send_if_modified(|settings| {
            let before = settings.clone();
            edit(settings);
            *settings != before
        });
        if changed {
            self.inner
                .revision
                .send_modify(|revision| *revision = revision.wrapping_add(1));
            tracing::trace!("preferences changed");
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<UiSettings> {
        self.inner.current.subscribe()
    }

    pub(crate) fn spawn_saver(&self, tasks: &BackgroundTasks) {
        let store = self.clone();
        let changes = self.subscribe();
        tasks.spawn_cancellable(SAVER_TASK, move |cancel| {
            store.save_on_change(changes, cancel)
        });
    }

    async fn save_on_change(
        self,
        mut changes: watch::Receiver<UiSettings>,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                () = cancel.cancelled() => break,
                outcome = changes.changed() => {
                    if outcome.is_err() {
                        break;
                    }
                }
            }
            tokio::select! {
                () = cancel.cancelled() => break,
                () = tokio::time::sleep(SAVE_DELAY) => {}
            }
            changes.mark_unchanged();
            let writer = self.clone();
            match OwnedTask::spawn_blocking(WRITER_TASK, move || writer.flush())
                .join()
                .await
            {
                Ok(Ok(())) => {},
                Ok(Err(error)) => tracing::warn!(error = %error, "preferences not saved"),
                Err(error) => tracing::warn!(error = %error, "preferences writer stopped"),
            }
        }
        if let Err(error) = self.flush() {
            tracing::warn!(error = %error, "preferences not saved at shutdown");
        }
    }

    pub fn flush(&self) -> Result<(), SettingsError> {
        let bytes = encode(&self.get())?;
        sp_secure::write_private(&self.inner.path, &bytes).map_err(SettingsError::Write)
    }
}

pub fn encode(settings: &UiSettings) -> Result<Vec<u8>, SettingsError> {
    serde_json::to_vec_pretty(settings).map_err(SettingsError::Encode)
}

pub fn decode(bytes: &[u8]) -> Result<UiSettings, serde_json::Error> {
    serde_json::from_slice(bytes)
}

fn read(path: &Path) -> UiSettings {
    match sp_secure::read_optional(path) {
        Ok(Some(bytes)) => match decode(&bytes) {
            Ok(settings) => settings,
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    line = error.line(),
                    column = error.column(),
                    "preferences do not decode; using defaults"
                );
                UiSettings::default()
            },
        },
        Ok(None) => UiSettings::default(),
        Err(error) => {
            tracing::warn!(path = %path.display(), error = %error, "preferences unreadable; using defaults");
            UiSettings::default()
        },
    }
}

impl SettingsAccess for SettingsStore {
    // JSON: an extension's own preferences object, stored as the extension wrote
    // it.
    fn read(&self, extension: &ExtensionId) -> Option<Map<String, Value>> {
        self.inner
            .current
            .borrow()
            .extensions
            .get(extension)
            .cloned()
    }

    // JSON: an extension's own preferences object, stored as the extension wrote
    // it.
    fn write(&self, extension: &ExtensionId, values: Map<String, Value>) {
        self.update(|settings| {
            settings.extensions.insert(extension.clone(), values);
        });
    }

    fn changes(&self) -> watch::Receiver<u64> {
        self.inner.revision.subscribe()
    }
}
