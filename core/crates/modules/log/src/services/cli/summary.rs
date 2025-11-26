use crate::services::cli::{
    display::{CollectionDisplay, Display, DisplayUtils, StatusDisplay},
    theme::{EmphasisType, ItemStatus, MessageLevel, Theme},
};
#[derive(Debug)]
pub struct ValidationSummary {
    pub valid: Vec<(String, String)>,
    pub installed: Vec<String>,
    pub updated: Vec<String>,
    pub schemas_applied: Vec<String>,
    pub seeds_applied: Vec<String>,
    pub disabled: Vec<String>,
}

impl ValidationSummary {
    pub const fn new() -> Self {
        Self {
            valid: Vec::new(),
            installed: Vec::new(),
            updated: Vec::new(),
            schemas_applied: Vec::new(),
            seeds_applied: Vec::new(),
            disabled: Vec::new(),
        }
    }

    pub fn add_valid(&mut self, name: String, version: String) {
        self.valid.push((name, version));
    }

    pub fn add_installed(&mut self, name: String) {
        self.installed.push(name);
    }

    pub fn add_updated(&mut self, name: String) {
        self.updated.push(name);
    }

    pub fn add_schema_applied(&mut self, name: String) {
        self.schemas_applied.push(name);
    }

    pub fn add_seed_applied(&mut self, name: String) {
        self.seeds_applied.push(name);
    }

    pub fn add_disabled(&mut self, name: String) {
        self.disabled.push(name);
    }

    pub fn total_active(&self) -> usize {
        self.valid.len() + self.installed.len() + self.updated.len()
    }

    pub fn has_changes(&self) -> bool {
        !self.installed.is_empty()
            || !self.updated.is_empty()
            || !self.schemas_applied.is_empty()
            || !self.seeds_applied.is_empty()
    }
}

impl Display for ValidationSummary {
    fn display(&self) {
        DisplayUtils::section_header("Module Validation Summary");

        if !self.valid.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .valid
                .iter()
                .map(|(name, version)| {
                    let detail = format!("v{version}");
                    StatusDisplay::new(ItemStatus::Valid, name).with_detail(detail)
                })
                .collect();

            let collection = CollectionDisplay::new("Valid modules", displays);
            collection.display();
        }

        if !self.installed.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .installed
                .iter()
                .map(|name| StatusDisplay::new(ItemStatus::Applied, name))
                .collect();

            let collection = CollectionDisplay::new("Newly installed", displays);
            collection.display();
        }

        if !self.updated.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .updated
                .iter()
                .map(|name| StatusDisplay::new(ItemStatus::Applied, name))
                .collect();

            let collection = CollectionDisplay::new("Updated modules", displays);
            collection.display();
        }

        if !self.schemas_applied.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .schemas_applied
                .iter()
                .map(|name| StatusDisplay::new(ItemStatus::Applied, name))
                .collect();

            let collection = CollectionDisplay::new("Schemas applied", displays);
            collection.display();
        }

        if !self.seeds_applied.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .seeds_applied
                .iter()
                .map(|name| StatusDisplay::new(ItemStatus::Applied, name))
                .collect();

            let collection = CollectionDisplay::new("Seeds applied", displays);
            collection.display();
        }

        if !self.disabled.is_empty() {
            let displays: Vec<StatusDisplay> = self
                .disabled
                .iter()
                .map(|name| StatusDisplay::new(ItemStatus::Disabled, name).with_detail("disabled"))
                .collect();

            let collection = CollectionDisplay::new("Disabled modules", displays);
            collection.display();
        }

        let total_active = self.total_active();
        if total_active > 0 {
            println!(
                "\n{} {} active modules ready",
                Theme::icon(MessageLevel::Success),
                Theme::color(&total_active.to_string(), EmphasisType::Bold)
            );
        }
    }
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub operation: String,
    pub success: bool,
    pub message: Option<String>,
    pub details: Vec<String>,
}

impl OperationResult {
    pub fn success(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            success: true,
            message: None,
            details: Vec::new(),
        }
    }

    pub fn failure(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            success: false,
            message: Some(message.into()),
            details: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    #[must_use]
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

impl Display for OperationResult {
    fn display(&self) {
        let level = if self.success {
            MessageLevel::Success
        } else {
            MessageLevel::Error
        };

        let base_message = self.message.as_ref().map_or_else(
            || self.operation.clone(),
            |msg| format!("{}: {msg}", self.operation),
        );

        DisplayUtils::message(level, &base_message);

        for detail in &self.details {
            let colored = Theme::color(detail, EmphasisType::Dim);
            println!("  • {colored}");
        }
    }
}

#[derive(Debug)]
pub struct ProgressSummary {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub operation_name: String,
}

impl ProgressSummary {
    pub fn new(operation_name: impl Into<String>, total: usize) -> Self {
        Self {
            total,
            completed: 0,
            failed: 0,
            operation_name: operation_name.into(),
        }
    }

    pub fn add_success(&mut self) {
        self.completed += 1;
    }

    pub fn add_failure(&mut self) {
        self.failed += 1;
    }

    pub const fn is_complete(&self) -> bool {
        self.completed + self.failed >= self.total
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.completed as f64 / self.total as f64
        }
    }
}

impl Display for ProgressSummary {
    fn display(&self) {
        let status = if self.failed == 0 {
            MessageLevel::Success
        } else if self.completed > 0 {
            MessageLevel::Warning
        } else {
            MessageLevel::Error
        };

        let message = if self.failed > 0 {
            format!(
                "{}: {}/{} completed, {} failed",
                self.operation_name, self.completed, self.total, self.failed
            )
        } else {
            format!(
                "{}: {}/{} completed",
                self.operation_name, self.completed, self.total
            )
        };

        DisplayUtils::message(status, &message);
    }
}
