use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;

use crate::services::cli::display::{CollectionDisplay, Display, DisplayUtils, StatusDisplay};
use crate::services::cli::theme::MessageLevel;
#[derive(Debug, Copy, Clone)]
pub struct Prompts;

impl Prompts {
    pub fn confirm(message: &str, default: bool) -> Result<bool> {
        // Check for non-interactive mode
        if std::env::var("SYSTEMPROMPT_NON_INTERACTIVE").is_ok() {
            println!(
                "{} (non-interactive: {})",
                message,
                if default { "yes" } else { "no" }
            );
            return Ok(default);
        }

        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .default(default)
            .interact()?;
        Ok(confirmation)
    }

    pub fn confirm_schemas() -> Result<bool> {
        Self::confirm("Apply these schemas?", true)
    }

    pub fn confirm_seeds() -> Result<bool> {
        Self::confirm("Apply these seeds?", true)
    }

    pub fn confirm_install(modules: &[String]) -> Result<bool> {
        if modules.is_empty() {
            return Ok(false);
        }

        DisplayUtils::section_header("New modules found");

        let displays: Vec<StatusDisplay> = modules
            .iter()
            .map(|name| {
                StatusDisplay::new(crate::services::cli::theme::ItemStatus::Pending, name)
                    .with_detail("ready to install")
            })
            .collect();

        let collection = CollectionDisplay::new("Modules", displays).without_count();
        collection.display();

        Self::confirm("Install these modules?", false)
    }

    pub fn confirm_update(updates: &[(String, String, String)]) -> Result<bool> {
        if updates.is_empty() {
            return Ok(false);
        }

        DisplayUtils::section_header("Module updates available");

        let displays: Vec<StatusDisplay> = updates
            .iter()
            .map(|(name, old, new)| {
                let detail = format!("{old} → {new}");
                StatusDisplay::new(crate::services::cli::theme::ItemStatus::Pending, name)
                    .with_detail(detail)
            })
            .collect();

        let collection = CollectionDisplay::new("Updates", displays).without_count();
        collection.display();

        Self::confirm("Update these modules?", false)
    }

    pub fn confirm_with_context<T: Display>(
        context_items: &[T],
        context_title: &str,
        question: &str,
        default: bool,
    ) -> Result<bool> {
        if !context_items.is_empty() {
            DisplayUtils::section_header(context_title);
            for item in context_items {
                item.display();
            }
            println!();
        }

        Self::confirm(question, default)
    }
}

pub struct PromptBuilder {
    title: Option<String>,
    message: String,
    default: bool,
    show_context: Vec<Box<dyn Display>>,
}

impl std::fmt::Debug for PromptBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptBuilder")
            .field("title", &self.title)
            .field("message", &self.message)
            .field("default", &self.default)
            .field(
                "show_context",
                &format!("[{} items]", self.show_context.len()),
            )
            .finish()
    }
}

impl PromptBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            title: None,
            message: message.into(),
            default: false,
            show_context: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub const fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    #[must_use]
    pub fn with_context<T: Display + 'static>(mut self, item: T) -> Self {
        self.show_context.push(Box::new(item));
        self
    }

    pub fn confirm(self) -> Result<bool> {
        if let Some(title) = &self.title {
            DisplayUtils::section_header(title);
        }

        for item in &self.show_context {
            item.display();
        }

        if !self.show_context.is_empty() {
            println!();
        }

        Prompts::confirm(&self.message, self.default)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct QuickPrompts;

impl QuickPrompts {
    pub fn yes_no(question: &str) -> Result<bool> {
        Prompts::confirm(question, false)
    }

    pub fn yes_no_default_yes(question: &str) -> Result<bool> {
        Prompts::confirm(question, true)
    }

    pub fn continue_or_abort(action: &str) -> Result<bool> {
        let message = format!("Continue with {action}?");
        Prompts::confirm(&message, false)
    }

    pub fn dangerous_action(action: &str) -> Result<bool> {
        let warning = format!("This will {action}");
        DisplayUtils::message(MessageLevel::Warning, &warning);
        Prompts::confirm("Are you sure?", false)
    }
}
