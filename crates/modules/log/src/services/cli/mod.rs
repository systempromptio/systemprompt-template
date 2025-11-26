pub mod display;
pub mod module;
pub mod prompts;
pub mod summary;
pub mod theme;

pub use display::{CollectionDisplay, Display, DisplayUtils, ModuleItemDisplay, StatusDisplay};
pub use module::{BatchModuleOperations, ModuleDisplay, ModuleInstall, ModuleUpdate};
pub use prompts::{PromptBuilder, Prompts, QuickPrompts};
pub use summary::{OperationResult, ProgressSummary, ValidationSummary};
pub use theme::{
    ActionType, Colors, EmphasisType, IconType, Icons, ItemStatus, MessageLevel, ModuleType, Theme,
};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct CliService;

impl CliService {
    pub fn success(message: &str) {
        DisplayUtils::message(MessageLevel::Success, message);
    }

    pub fn warning(message: &str) {
        DisplayUtils::message(MessageLevel::Warning, message);
    }

    pub fn error(message: &str) {
        DisplayUtils::message(MessageLevel::Error, message);
    }

    pub fn info(message: &str) {
        DisplayUtils::message(MessageLevel::Info, message);
    }

    pub fn debug(message: &str) {
        let debug_msg = format!("DEBUG: {message}");
        DisplayUtils::message(MessageLevel::Info, &debug_msg);
    }

    pub fn verbose(message: &str) {
        DisplayUtils::message(MessageLevel::Info, message);
    }

    pub fn fatal(message: &str, exit_code: i32) -> ! {
        let fatal_msg = format!("FATAL: {message}");
        DisplayUtils::message(MessageLevel::Error, &fatal_msg);
        std::process::exit(exit_code);
    }

    pub fn section(title: &str) {
        DisplayUtils::section_header(title);
    }

    pub fn json<T: Serialize>(value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{json}");
        }
    }

    pub fn json_compact<T: Serialize>(value: &T) {
        if let Ok(json) = serde_json::to_string(value) {
            println!("{json}");
        }
    }

    pub fn yaml<T: Serialize>(value: &T) {
        if let Ok(yaml) = serde_yaml::to_string(value) {
            print!("{yaml}");
        }
    }

    pub fn key_value(label: &str, value: &str) {
        println!(
            "{}: {}",
            Theme::color(label, EmphasisType::Bold),
            Theme::color(value, EmphasisType::Highlight)
        );
    }

    pub fn status_line(label: &str, value: &str, status: ItemStatus) {
        println!(
            "{} {}: {}",
            Theme::icon(status),
            Theme::color(label, EmphasisType::Bold),
            Theme::color(value, status)
        );
    }

    pub fn spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn progress_bar(total: u64) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#>-"),
        );
        pb
    }

    pub fn timed<F, R>(label: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        let duration_secs = duration.as_secs_f64();
        let info_msg = format!("{label} completed in {duration_secs:.2}s");
        Self::info(&info_msg);
        result
    }

    pub fn prompt_schemas(module_name: &str, schemas: &[(String, String)]) -> Result<bool> {
        ModuleDisplay::prompt_apply_schemas(module_name, schemas)
    }

    pub fn prompt_seeds(module_name: &str, seeds: &[(String, String)]) -> Result<bool> {
        ModuleDisplay::prompt_apply_seeds(module_name, seeds)
    }

    pub fn prompt_install(modules: &[String]) -> Result<bool> {
        Prompts::confirm_install(modules)
    }

    pub fn prompt_update(updates: &[(String, String, String)]) -> Result<bool> {
        Prompts::confirm_update(updates)
    }

    pub fn confirm(question: &str) -> Result<bool> {
        Prompts::confirm(question, false)
    }

    pub fn confirm_default_yes(question: &str) -> Result<bool> {
        Prompts::confirm(question, true)
    }

    pub fn display_validation_summary(
        valid: &[(String, String)],
        installed: &[String],
        updated: &[String],
        schemas_applied: &[String],
        seeds_applied: &[String],
        disabled: &[String],
    ) {
        let mut summary = ValidationSummary::new();

        for (name, version) in valid {
            summary.add_valid(name.clone(), version.clone());
        }
        for name in installed {
            summary.add_installed(name.clone());
        }
        for name in updated {
            summary.add_updated(name.clone());
        }
        for name in schemas_applied {
            summary.add_schema_applied(name.clone());
        }
        for name in seeds_applied {
            summary.add_seed_applied(name.clone());
        }
        for name in disabled {
            summary.add_disabled(name.clone());
        }

        summary.display();
    }

    pub fn display_result(result: &OperationResult) {
        result.display();
    }

    pub fn display_progress(progress: &ProgressSummary) {
        progress.display();
    }

    pub fn prompt_builder(message: &str) -> PromptBuilder {
        PromptBuilder::new(message)
    }

    pub fn collection<T: Display>(title: &str, items: Vec<T>) -> CollectionDisplay<T> {
        CollectionDisplay::new(title, items)
    }

    pub fn module_status(module_name: &str, message: &str) {
        DisplayUtils::module_status(module_name, message);
    }

    pub fn relationship(from: &str, to: &str, status: ItemStatus, module_type: ModuleType) {
        DisplayUtils::relationship(module_type, from, to, status);
    }

    pub fn item(status: ItemStatus, name: &str, detail: Option<&str>) {
        DisplayUtils::item(status, name, detail);
    }

    pub fn batch_install(modules: &[ModuleInstall]) -> Result<bool> {
        BatchModuleOperations::prompt_install_multiple(modules)
    }

    pub fn batch_update(updates: &[ModuleUpdate]) -> Result<bool> {
        BatchModuleOperations::prompt_update_multiple(updates)
    }

    /// Display a table with headers and rows
    pub fn table(headers: &[&str], rows: &[Vec<String>]) {
        if rows.is_empty() {
            return;
        }

        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print top border
        print!("┌");
        for (i, &width) in widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < widths.len() - 1 {
                print!("┬");
            }
        }
        println!("┐");

        // Print headers
        print!("│");
        for (i, (header, &width)) in headers.iter().zip(widths.iter()).enumerate() {
            print!(" {header:<width$} ");
            if i < widths.len() - 1 {
                print!("│");
            }
        }
        println!("│");

        // Print header separator
        print!("├");
        for (i, &width) in widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < widths.len() - 1 {
                print!("┼");
            }
        }
        println!("┤");

        // Print rows
        for row in rows {
            print!("│");
            for (i, (cell, &width)) in row.iter().zip(widths.iter()).enumerate() {
                let truncated = if cell.len() > width {
                    &cell[..width.saturating_sub(3)]
                } else {
                    cell
                };
                print!(" {truncated:<width$} ");
                if i < widths.len() - 1 {
                    print!("│");
                }
            }
            println!("│");
        }

        // Print bottom border
        print!("└");
        for (i, &width) in widths.iter().enumerate() {
            print!("{}", "─".repeat(width + 2));
            if i < widths.len() - 1 {
                print!("┴");
            }
        }
        println!("┘");
    }
}

#[macro_export]
macro_rules! cli_success {
    ($($arg:tt)*) => {
        $crate::services::cli::CliService::success(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! cli_warning {
    ($($arg:tt)*) => {
        $crate::services::cli::CliService::warning(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! cli_error {
    ($($arg:tt)*) => {
        $crate::services::cli::CliService::error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! cli_info {
    ($($arg:tt)*) => {
        $crate::services::cli::CliService::info(&format!($($arg)*))
    };
}
