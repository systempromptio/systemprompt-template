use crate::services::cli::theme::{
    ActionType, EmphasisType, IconType, ItemStatus, MessageLevel, ModuleType, Theme,
};
pub trait Display {
    fn display(&self);
}

pub trait DetailedDisplay {
    fn display_summary(&self);
    fn display_details(&self);
}

#[derive(Debug, Copy, Clone)]
pub struct DisplayUtils;

impl DisplayUtils {
    pub fn message(level: MessageLevel, text: &str) {
        println!("{} {}", Theme::icon(level), Theme::color(text, level));
    }

    pub fn section_header(title: &str) {
        println!("\n{}", Theme::color(title, EmphasisType::Underlined));
    }

    pub fn item(icon_type: impl Into<IconType>, name: &str, detail: Option<&str>) {
        match detail {
            Some(detail) => println!(
                "   {} {} {}",
                Theme::icon(icon_type),
                Theme::color(name, EmphasisType::Bold),
                Theme::color(detail, EmphasisType::Dim)
            ),
            None => println!(
                "   {} {}",
                Theme::icon(icon_type),
                Theme::color(name, EmphasisType::Bold)
            ),
        }
    }

    pub fn relationship(icon_type: impl Into<IconType>, from: &str, to: &str, status: ItemStatus) {
        println!(
            "   {} {} {} {} {}",
            Theme::icon(icon_type),
            Theme::color(from, EmphasisType::Highlight),
            Theme::icon(ActionType::Arrow),
            Theme::color(to, status),
            Theme::color(&format!("({})", status_text(status)), EmphasisType::Dim)
        );
    }

    pub fn module_status(module_name: &str, message: &str) {
        let module_label = format!("Module: {module_name}");
        println!(
            "{} {} {}",
            Theme::icon(ModuleType::Module),
            Theme::color(&module_label, EmphasisType::Highlight),
            Theme::color(message, EmphasisType::Dim)
        );
    }

    pub fn count_message(level: MessageLevel, count: usize, item_type: &str) {
        let count_label = format!("{} {item_type}", count_text(count, item_type));
        let count_str = count.to_string();
        println!(
            "   {} {}: {}",
            Theme::icon(level),
            count_label,
            Theme::color(&count_str, level)
        );
    }
}

#[derive(Debug)]
pub struct StatusDisplay {
    pub status: ItemStatus,
    pub name: String,
    pub detail: Option<String>,
}

impl StatusDisplay {
    pub fn new(status: ItemStatus, name: impl Into<String>) -> Self {
        Self {
            status,
            name: name.into(),
            detail: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl Display for StatusDisplay {
    fn display(&self) {
        DisplayUtils::item(self.status, &self.name, self.detail.as_deref());
    }
}

#[derive(Debug)]
pub struct ModuleItemDisplay {
    pub module_type: ModuleType,
    pub file: String,
    pub target: String,
    pub status: ItemStatus,
}

impl ModuleItemDisplay {
    pub fn new(
        module_type: ModuleType,
        file: impl Into<String>,
        target: impl Into<String>,
        status: ItemStatus,
    ) -> Self {
        Self {
            module_type,
            file: file.into(),
            target: target.into(),
            status,
        }
    }
}

impl Display for ModuleItemDisplay {
    fn display(&self) {
        DisplayUtils::relationship(self.module_type, &self.file, &self.target, self.status);
    }
}

#[derive(Debug)]
pub struct CollectionDisplay<T: Display> {
    pub title: String,
    pub items: Vec<T>,
    pub show_count: bool,
}

impl<T: Display> CollectionDisplay<T> {
    pub fn new(title: impl Into<String>, items: Vec<T>) -> Self {
        Self {
            title: title.into(),
            items,
            show_count: true,
        }
    }

    #[must_use]
    pub const fn without_count(mut self) -> Self {
        self.show_count = false;
        self
    }
}

impl<T: Display> Display for CollectionDisplay<T> {
    fn display(&self) {
        if self.show_count && !self.items.is_empty() {
            println!(
                "\n{} {}:",
                Theme::color(&self.title, EmphasisType::Bold),
                Theme::color(&format!("({})", self.items.len()), EmphasisType::Dim)
            );
        } else if !self.items.is_empty() {
            println!("\n{}:", Theme::color(&self.title, EmphasisType::Bold));
        }

        for item in &self.items {
            item.display();
        }
    }
}

const fn status_text(status: ItemStatus) -> &'static str {
    match status {
        ItemStatus::Missing => "missing",
        ItemStatus::Applied => "applied",
        ItemStatus::Failed => "failed",
        ItemStatus::Valid => "valid",
        ItemStatus::Disabled => "disabled",
        ItemStatus::Pending => "pending",
    }
}

fn count_text(count: usize, item_type: &str) -> &'static str {
    if count == 1 {
        match item_type {
            "schemas" => "Missing schema",
            "seeds" => "Missing seed",
            "modules" => "New module",
            _ => "Missing item",
        }
    } else {
        match item_type {
            "schemas" => "Missing schemas",
            "seeds" => "Missing seeds",
            "modules" => "New modules",
            _ => "Missing items",
        }
    }
}
