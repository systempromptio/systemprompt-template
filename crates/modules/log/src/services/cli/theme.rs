use console::{style, Emoji, StyledObject};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    Missing,
    Applied,
    Failed,
    Valid,
    Disabled,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Schema,
    Seed,
    Module,
    Configuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Copy, Clone)]
pub struct Icons;

impl Icons {
    pub const CHECKMARK: Emoji<'static, 'static> = Emoji("✓", "✓");
    pub const WARNING: Emoji<'static, 'static> = Emoji("⚠", "!");
    pub const ERROR: Emoji<'static, 'static> = Emoji("✗", "X");
    pub const INFO: Emoji<'static, 'static> = Emoji("ℹ", "i");

    pub const PACKAGE: Emoji<'static, 'static> = Emoji("📦", "[MOD]");
    pub const SCHEMA: Emoji<'static, 'static> = Emoji("📄", "[SCHEMA]");
    pub const SEED: Emoji<'static, 'static> = Emoji("🌱", "[SEED]");
    pub const CONFIG: Emoji<'static, 'static> = Emoji("⚙", "[CONFIG]");

    pub const ARROW: Emoji<'static, 'static> = Emoji("→", "->");
    pub const UPDATE: Emoji<'static, 'static> = Emoji("🔄", "[UPDATE]");
    pub const INSTALL: Emoji<'static, 'static> = Emoji("📥", "[INSTALL]");
    pub const PAUSE: Emoji<'static, 'static> = Emoji("⏸", "[PAUSED]");

    pub const fn for_module_type(module_type: ModuleType) -> Emoji<'static, 'static> {
        match module_type {
            ModuleType::Schema => Self::SCHEMA,
            ModuleType::Seed => Self::SEED,
            ModuleType::Module => Self::PACKAGE,
            ModuleType::Configuration => Self::CONFIG,
        }
    }

    pub const fn for_status(status: ItemStatus) -> Emoji<'static, 'static> {
        match status {
            ItemStatus::Valid | ItemStatus::Applied => Self::CHECKMARK,
            ItemStatus::Missing | ItemStatus::Pending => Self::WARNING,
            ItemStatus::Failed => Self::ERROR,
            ItemStatus::Disabled => Self::PAUSE,
        }
    }

    pub const fn for_message_level(level: MessageLevel) -> Emoji<'static, 'static> {
        match level {
            MessageLevel::Success => Self::CHECKMARK,
            MessageLevel::Warning => Self::WARNING,
            MessageLevel::Error => Self::ERROR,
            MessageLevel::Info => Self::INFO,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Colors;

impl Colors {
    pub fn success<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).green()
    }

    pub fn warning<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).yellow()
    }

    pub fn error<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).red()
    }

    pub fn info<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).cyan()
    }

    pub fn highlight<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).bold().cyan()
    }

    pub fn dim<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).dim()
    }

    pub fn bold<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).bold()
    }

    pub fn underlined<D: std::fmt::Display>(text: D) -> StyledObject<D> {
        style(text).bold().underlined()
    }

    pub fn for_status<D: std::fmt::Display>(text: D, status: ItemStatus) -> StyledObject<D> {
        match status {
            ItemStatus::Valid | ItemStatus::Applied => Self::success(text),
            ItemStatus::Missing | ItemStatus::Pending => Self::warning(text),
            ItemStatus::Failed => Self::error(text),
            ItemStatus::Disabled => Self::dim(text),
        }
    }

    pub fn for_message_level<D: std::fmt::Display>(
        text: D,
        level: MessageLevel,
    ) -> StyledObject<D> {
        match level {
            MessageLevel::Success => Self::success(text),
            MessageLevel::Warning => Self::warning(text),
            MessageLevel::Error => Self::error(text),
            MessageLevel::Info => Self::info(text),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme;

impl Theme {
    pub fn icon(icon_type: impl Into<IconType>) -> Emoji<'static, 'static> {
        match icon_type.into() {
            IconType::Status(status) => Icons::for_status(status),
            IconType::Module(module_type) => Icons::for_module_type(module_type),
            IconType::Message(level) => Icons::for_message_level(level),
            IconType::Action(action) => match action {
                ActionType::Install => Icons::INSTALL,
                ActionType::Update => Icons::UPDATE,
                ActionType::Arrow => Icons::ARROW,
            },
        }
    }

    pub fn color<D: std::fmt::Display>(
        text: D,
        color_type: impl Into<ColorType>,
    ) -> StyledObject<D> {
        match color_type.into() {
            ColorType::Status(status) => Colors::for_status(text, status),
            ColorType::Message(level) => Colors::for_message_level(text, level),
            ColorType::Emphasis(emphasis) => match emphasis {
                EmphasisType::Highlight => Colors::highlight(text),
                EmphasisType::Dim => Colors::dim(text),
                EmphasisType::Bold => Colors::bold(text),
                EmphasisType::Underlined => Colors::underlined(text),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IconType {
    Status(ItemStatus),
    Module(ModuleType),
    Message(MessageLevel),
    Action(ActionType),
}

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Status(ItemStatus),
    Message(MessageLevel),
    Emphasis(EmphasisType),
}

#[derive(Debug, Clone, Copy)]
pub enum ActionType {
    Install,
    Update,
    Arrow,
}

#[derive(Debug, Clone, Copy)]
pub enum EmphasisType {
    Highlight,
    Dim,
    Bold,
    Underlined,
}

impl From<ItemStatus> for IconType {
    fn from(status: ItemStatus) -> Self {
        Self::Status(status)
    }
}

impl From<ModuleType> for IconType {
    fn from(module_type: ModuleType) -> Self {
        Self::Module(module_type)
    }
}

impl From<MessageLevel> for IconType {
    fn from(level: MessageLevel) -> Self {
        Self::Message(level)
    }
}

impl From<ActionType> for IconType {
    fn from(action: ActionType) -> Self {
        Self::Action(action)
    }
}

impl From<ItemStatus> for ColorType {
    fn from(status: ItemStatus) -> Self {
        Self::Status(status)
    }
}

impl From<MessageLevel> for ColorType {
    fn from(level: MessageLevel) -> Self {
        Self::Message(level)
    }
}

impl From<EmphasisType> for ColorType {
    fn from(emphasis: EmphasisType) -> Self {
        Self::Emphasis(emphasis)
    }
}
