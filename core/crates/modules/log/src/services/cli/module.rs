use crate::services::cli::display::{CollectionDisplay, Display, DisplayUtils, ModuleItemDisplay};
use crate::services::cli::prompts::Prompts;
use crate::services::cli::theme::{ItemStatus, MessageLevel, ModuleType, Theme};
use anyhow::Result;
#[derive(Debug, Copy, Clone)]
pub struct ModuleDisplay;

impl ModuleDisplay {
    pub fn missing_schemas(module_name: &str, schemas: &[(String, String)]) {
        if schemas.is_empty() {
            return;
        }

        DisplayUtils::module_status(
            module_name,
            &format!("{} schemas need application", schemas.len()),
        );

        DisplayUtils::count_message(MessageLevel::Warning, schemas.len(), "schemas");

        let items: Vec<ModuleItemDisplay> = schemas
            .iter()
            .map(|(file, table)| {
                ModuleItemDisplay::new(ModuleType::Schema, file, table, ItemStatus::Missing)
            })
            .collect();

        for item in &items {
            item.display();
        }
    }

    pub fn missing_seeds(module_name: &str, seeds: &[(String, String)]) {
        if seeds.is_empty() {
            return;
        }

        DisplayUtils::module_status(
            module_name,
            &format!("{} seeds need application", seeds.len()),
        );

        DisplayUtils::count_message(MessageLevel::Warning, seeds.len(), "seeds");

        let items: Vec<ModuleItemDisplay> = seeds
            .iter()
            .map(|(file, table)| {
                ModuleItemDisplay::new(ModuleType::Seed, file, table, ItemStatus::Missing)
            })
            .collect();

        for item in &items {
            item.display();
        }
    }

    pub fn prompt_apply_schemas(module_name: &str, schemas: &[(String, String)]) -> Result<bool> {
        if schemas.is_empty() {
            return Ok(false);
        }

        Self::missing_schemas(module_name, schemas);
        println!();
        Prompts::confirm_schemas()
    }

    pub fn prompt_apply_seeds(module_name: &str, seeds: &[(String, String)]) -> Result<bool> {
        if seeds.is_empty() {
            return Ok(false);
        }

        Self::missing_seeds(module_name, seeds);
        println!();
        Prompts::confirm_seeds()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleUpdate {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub changes: Vec<String>,
}

impl ModuleUpdate {
    pub fn new(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            old_version: old_version.into(),
            new_version: new_version.into(),
            changes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_change(mut self, change: impl Into<String>) -> Self {
        self.changes.push(change.into());
        self
    }

    #[must_use]
    pub fn with_changes(mut self, changes: Vec<String>) -> Self {
        self.changes = changes;
        self
    }
}

impl Display for ModuleUpdate {
    fn display(&self) {
        println!(
            "   {} {} {}",
            Theme::icon(crate::services::cli::theme::ActionType::Update),
            Theme::color(&self.name, crate::services::cli::theme::EmphasisType::Bold),
            Theme::color(
                &format!("{} → {}", self.old_version, self.new_version),
                crate::services::cli::theme::EmphasisType::Dim
            )
        );

        for change in &self.changes {
            println!(
                "     • {}",
                Theme::color(change, crate::services::cli::theme::EmphasisType::Dim)
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleInstall {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

impl ModuleInstall {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
        }
    }

    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Display for ModuleInstall {
    fn display(&self) {
        let detail = self.description.as_ref().map_or_else(
            || format!("v{}", self.version),
            |desc| format!("v{} - {}", self.version, desc),
        );

        println!(
            "   {} {} {}",
            Theme::icon(crate::services::cli::theme::ActionType::Install),
            Theme::color(&self.name, crate::services::cli::theme::EmphasisType::Bold),
            Theme::color(&detail, crate::services::cli::theme::EmphasisType::Dim)
        );
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BatchModuleOperations;

impl BatchModuleOperations {
    pub fn prompt_install_multiple(modules: &[ModuleInstall]) -> Result<bool> {
        if modules.is_empty() {
            return Ok(false);
        }

        let collection =
            CollectionDisplay::new("New modules available for installation", modules.to_vec());
        collection.display();

        Prompts::confirm("Install all these modules?", false)
    }

    pub fn prompt_update_multiple(updates: &[ModuleUpdate]) -> Result<bool> {
        if updates.is_empty() {
            return Ok(false);
        }

        let collection = CollectionDisplay::new("Module updates available", updates.to_vec());
        collection.display();

        Prompts::confirm("Update all these modules?", false)
    }
}
