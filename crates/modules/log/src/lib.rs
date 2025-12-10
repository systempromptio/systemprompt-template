#![allow(clippy::must_use_candidate)]

pub mod models;
pub mod repository;
pub mod services;

pub use models::{LogEntry, LogLevel};
pub use repository::{AnalyticsEvent, AnalyticsRepository, LoggingRepository};
pub use services::{format_log, format_log_owned, CliService, LogContext, LogOutput, LogService};
pub use tracing::{info, warn};

pub fn init(_level: LogLevel) {
    tracing_subscriber::fmt::init();
}
