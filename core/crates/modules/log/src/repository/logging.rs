#![allow(clippy::print_stdout, clippy::print_stderr)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use systemprompt_core_database::DbPool;

use super::{
    cleanup_old_logs, clear_all_logs, create_log, delete_log_entries, delete_log_entry,
    get_log_by_id, get_logs_paginated, get_recent_logs, update_log_entry,
};
use crate::models::{LogEntry, LogLevel, LoggingError};

#[derive(Clone, Debug)]
pub struct LoggingRepository {
    db_pool: DbPool,
    terminal_output: bool,
    db_output: bool,
}

impl LoggingRepository {
    #[must_use]
    pub const fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            terminal_output: true,
            db_output: false,
        }
    }

    #[must_use]
    pub const fn with_terminal(mut self, enabled: bool) -> Self {
        self.terminal_output = enabled;
        self
    }

    #[must_use]
    pub const fn with_database(mut self, enabled: bool) -> Self {
        self.db_output = enabled;
        self
    }

    pub async fn log(&self, entry: LogEntry) -> Result<(), LoggingError> {
        entry.validate()?;

        if self.terminal_output {
            println!("{entry}");
        }

        if self.db_output {
            create_log(self.db_pool.as_ref(), &entry)
                .await
                .map_err(LoggingError::from)?;
        }

        Ok(())
    }

    pub async fn error(&self, module: &str, message: &str) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Error, module, message);
        self.log(entry).await
    }

    pub async fn error_with_metadata(
        &self,
        module: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Error, module, message).with_metadata(metadata);
        self.log(entry).await
    }

    pub async fn warn(&self, module: &str, message: &str) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Warn, module, message);
        self.log(entry).await
    }

    pub async fn warn_with_metadata(
        &self,
        module: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Warn, module, message).with_metadata(metadata);
        self.log(entry).await
    }

    pub async fn info(&self, module: &str, message: &str) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Info, module, message);
        self.log(entry).await
    }

    pub async fn info_with_metadata(
        &self,
        module: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Info, module, message).with_metadata(metadata);
        self.log(entry).await
    }

    pub async fn debug(&self, module: &str, message: &str) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Debug, module, message);
        self.log(entry).await
    }

    pub async fn debug_with_metadata(
        &self,
        module: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Debug, module, message).with_metadata(metadata);
        self.log(entry).await
    }

    pub async fn trace(&self, module: &str, message: &str) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Trace, module, message);
        self.log(entry).await
    }

    pub async fn trace_with_metadata(
        &self,
        module: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> Result<(), LoggingError> {
        let entry = LogEntry::new(LogLevel::Trace, module, message).with_metadata(metadata);
        self.log(entry).await
    }

    pub async fn get_recent_logs(&self, limit: i64) -> Result<Vec<LogEntry>, LoggingError> {
        get_recent_logs(self.db_pool.as_ref(), limit)
            .await
            .map_err(LoggingError::from)
    }

    pub async fn cleanup_old_logs(&self, older_than: DateTime<Utc>) -> Result<u64, LoggingError> {
        let rows_affected = cleanup_old_logs(self.db_pool.as_ref(), older_than)
            .await
            .map_err(LoggingError::from)?;
        Ok(rows_affected)
    }

    pub async fn clear_all_logs(&self) -> Result<u64, LoggingError> {
        let rows_affected = clear_all_logs(self.db_pool.as_ref())
            .await
            .map_err(LoggingError::from)?;
        Ok(rows_affected)
    }

    pub async fn get_log_by_id(&self, id: &str) -> Result<Option<LogEntry>, LoggingError> {
        if id.is_empty() {
            return Err(LoggingError::invalid_log_entry("id", "cannot be empty"));
        }
        get_log_by_id(self.db_pool.as_ref(), id)
            .await
            .map_err(LoggingError::from)
    }

    pub async fn update_log_entry(&self, id: &str, entry: &LogEntry) -> Result<bool, LoggingError> {
        if id.is_empty() {
            return Err(LoggingError::invalid_log_entry("id", "cannot be empty"));
        }
        entry.validate()?;
        update_log_entry(self.db_pool.as_ref(), id, entry)
            .await
            .map_err(LoggingError::from)
    }

    pub async fn delete_log_entry(&self, id: &str) -> Result<bool, LoggingError> {
        if id.is_empty() {
            return Err(LoggingError::invalid_log_entry("id", "cannot be empty"));
        }
        delete_log_entry(self.db_pool.as_ref(), id)
            .await
            .map_err(LoggingError::from)
    }

    pub async fn delete_log_entries(&self, ids: &[String]) -> Result<u64, LoggingError> {
        if ids.is_empty() {
            return Ok(0);
        }
        delete_log_entries(self.db_pool.as_ref(), ids)
            .await
            .map_err(LoggingError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_logs_paginated(
        &self,
        page: i32,
        per_page: i32,
        level_filter: Option<&str>,
        module_filter: Option<&str>,
        message_filter: Option<&str>,
    ) -> Result<(Vec<LogEntry>, i64), LoggingError> {
        if page < 1 || per_page < 1 {
            return Err(LoggingError::pagination_error(page, per_page));
        }

        if let Some(level) = level_filter {
            if level.parse::<LogLevel>().is_err() {
                return Err(LoggingError::filter_error("level", level));
            }
        }

        get_logs_paginated(
            self.db_pool.as_ref(),
            page,
            per_page,
            level_filter,
            module_filter,
            message_filter,
        )
        .await
        .map_err(LoggingError::from)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DisplayUtil;

impl DisplayUtil {
    pub fn display_table<T, F>(title: &str, items: Vec<T>, formatter: F)
    where
        F: Fn(&T) -> Vec<String>,
    {
        Self::print_header(title);

        for item in items {
            let lines = formatter(&item);
            for line in lines {
                println!("{line}");
            }
            Self::print_separator();
        }

        Self::print_footer();
    }

    pub fn success(message: &str) {
        println!("\u{2705} {message}");
    }

    pub fn error(message: &str) {
        eprintln!("\u{274c} {message}");
    }

    pub fn info(message: &str) {
        println!("\u{1f680} {message}");
    }

    pub fn warning(message: &str) {
        println!("\u{26a0}\u{fe0f} {message}");
    }

    pub fn status(label: &str, value: &str) {
        println!("   - {label}: {value}");
    }

    pub fn not_found(resource: &str, id: impl std::fmt::Display) {
        Self::error(&format!("{resource} {id} not found"));
    }

    fn print_header(title: &str) {
        let border = "\u{2500}".repeat(62);
        println!("\u{250c}{border}\u{2510}");
        println!("\u{2502}{title:^62}\u{2502}");
        println!("\u{251c}{border}\u{2524}");
    }

    fn print_separator() {
        let border = "\u{2500}".repeat(62);
        println!("\u{251c}{border}\u{2524}");
    }

    fn print_footer() {
        let border = "\u{2500}".repeat(62);
        println!("\u{2514}{border}\u{2518}");
    }
}

#[must_use]
pub fn format_server_status(is_enabled: bool) -> String {
    if is_enabled {
        format!("{} Enabled", "\u{2705}")
    } else {
        format!("{} Disabled", "\u{274c}")
    }
}

#[must_use]
pub fn format_component_counts(tool_count: i64, resource_count: i64, prompt_count: i64) -> String {
    format!("Components: {tool_count} tools, {resource_count} resources, {prompt_count} prompts")
}
