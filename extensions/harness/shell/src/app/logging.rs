//! The shell's log: a daily rolling file under the data root's `logs`
//! directory, named with the build's link scheme and kept for a week.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

const SUFFIX: &str = "log";
const KEEP_FILES: usize = 7;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LoggingError {
    #[error("the log directory {} could not be used", path.display())]
    Directory {
        path: std::path::PathBuf,
        #[source]
        source: tracing_appender::rolling::InitError,
    },
    #[error("a log subscriber was already installed")]
    Installed(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub(crate) fn init(directory: &Path, prefix: &str) -> Result<WorkerGuard, LoggingError> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(prefix)
        .filename_suffix(SUFFIX)
        .max_log_files(KEEP_FILES)
        .build(directory)
        .map_err(|source| LoggingError::Directory {
            path: directory.to_path_buf(),
            source,
        })?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(level)
        .with_target(true)
        .try_init()
        .map_err(LoggingError::Installed)?;
    Ok(guard)
}
