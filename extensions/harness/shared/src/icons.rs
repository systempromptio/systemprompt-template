//! The shell's own icon set: stroke icons on a 24 px grid, embedded in the
//! binary.
//!
//! [`IconName`] converts into the toolkit's icon type, so it works anywhere the
//! toolkit takes an icon (buttons, menu items, notifications). [`Assets`]
//! serves these icons and falls back to the toolkit's bundled ones; the shell
//! registers it with `Application::with_assets`.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::borrow::Cow;

use gpui_kit::component::{Icon as ToolkitIcon, IconNamed, Sizable as _};
use gpui_kit::{
    App, AssetSource, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Styled as _, Window,
};

use crate::theme::size;

const PREFIX: &str = "harness/icons/";

/// An icon of the shell's set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconName {
    Send,
    Stop,
    Plus,
    Minus,
    Folder,
    File,
    Terminal,
    Search,
    Settings,
    Check,
    X,
    ChevronUp,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    Alert,
    Info,
    Shield,
    Cloud,
    Sync,
    Key,
    User,
    Trash,
    Copy,
    Undo,
    Branch,
    Sparkle,
    ListTodo,
    Globe,
    Plug,
    Ellipsis,
    ExternalLink,
    CircleCheck,
    CircleX,
    Bell,
}

impl IconName {
    pub const ALL: [Self; 35] = [
        Self::Send,
        Self::Stop,
        Self::Plus,
        Self::Minus,
        Self::Folder,
        Self::File,
        Self::Terminal,
        Self::Search,
        Self::Settings,
        Self::Check,
        Self::X,
        Self::ChevronUp,
        Self::ChevronDown,
        Self::ChevronLeft,
        Self::ChevronRight,
        Self::Alert,
        Self::Info,
        Self::Shield,
        Self::Cloud,
        Self::Sync,
        Self::Key,
        Self::User,
        Self::Trash,
        Self::Copy,
        Self::Undo,
        Self::Branch,
        Self::Sparkle,
        Self::ListTodo,
        Self::Globe,
        Self::Plug,
        Self::Ellipsis,
        Self::ExternalLink,
        Self::CircleCheck,
        Self::CircleX,
        Self::Bell,
    ];

    #[must_use]
    pub const fn stem(self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::Stop => "stop",
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Folder => "folder",
            Self::File => "file",
            Self::Terminal => "terminal",
            Self::Search => "search",
            Self::Settings => "settings",
            Self::Check => "check",
            Self::X => "x",
            Self::ChevronUp => "chevron-up",
            Self::ChevronDown => "chevron-down",
            Self::ChevronLeft => "chevron-left",
            Self::ChevronRight => "chevron-right",
            Self::Alert => "alert",
            Self::Info => "info",
            Self::Shield => "shield",
            Self::Cloud => "cloud",
            Self::Sync => "sync",
            Self::Key => "key",
            Self::User => "user",
            Self::Trash => "trash",
            Self::Copy => "copy",
            Self::Undo => "undo",
            Self::Branch => "branch",
            Self::Sparkle => "sparkle",
            Self::ListTodo => "list-todo",
            Self::Globe => "globe",
            Self::Plug => "plug",
            Self::Ellipsis => "ellipsis",
            Self::ExternalLink => "external-link",
            Self::CircleCheck => "circle-check",
            Self::CircleX => "circle-x",
            Self::Bell => "bell",
        }
    }

    #[must_use]
    pub fn asset_path(self) -> SharedString {
        SharedString::from(format!("{PREFIX}{}.svg", self.stem()))
    }

    #[must_use]
    pub const fn svg(self) -> &'static [u8] {
        match self {
            Self::Send => include_bytes!("../assets/icons/send.svg"),
            Self::Stop => include_bytes!("../assets/icons/stop.svg"),
            Self::Plus => include_bytes!("../assets/icons/plus.svg"),
            Self::Minus => include_bytes!("../assets/icons/minus.svg"),
            Self::Folder => include_bytes!("../assets/icons/folder.svg"),
            Self::File => include_bytes!("../assets/icons/file.svg"),
            Self::Terminal => include_bytes!("../assets/icons/terminal.svg"),
            Self::Search => include_bytes!("../assets/icons/search.svg"),
            Self::Settings => include_bytes!("../assets/icons/settings.svg"),
            Self::Check => include_bytes!("../assets/icons/check.svg"),
            Self::X => include_bytes!("../assets/icons/x.svg"),
            Self::ChevronUp => include_bytes!("../assets/icons/chevron-up.svg"),
            Self::ChevronDown => include_bytes!("../assets/icons/chevron-down.svg"),
            Self::ChevronLeft => include_bytes!("../assets/icons/chevron-left.svg"),
            Self::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Self::Alert => include_bytes!("../assets/icons/alert.svg"),
            Self::Info => include_bytes!("../assets/icons/info.svg"),
            Self::Shield => include_bytes!("../assets/icons/shield.svg"),
            Self::Cloud => include_bytes!("../assets/icons/cloud.svg"),
            Self::Sync => include_bytes!("../assets/icons/sync.svg"),
            Self::Key => include_bytes!("../assets/icons/key.svg"),
            Self::User => include_bytes!("../assets/icons/user.svg"),
            Self::Trash => include_bytes!("../assets/icons/trash.svg"),
            Self::Copy => include_bytes!("../assets/icons/copy.svg"),
            Self::Undo => include_bytes!("../assets/icons/undo.svg"),
            Self::Branch => include_bytes!("../assets/icons/branch.svg"),
            Self::Sparkle => include_bytes!("../assets/icons/sparkle.svg"),
            Self::ListTodo => include_bytes!("../assets/icons/list-todo.svg"),
            Self::Globe => include_bytes!("../assets/icons/globe.svg"),
            Self::Plug => include_bytes!("../assets/icons/plug.svg"),
            Self::Ellipsis => include_bytes!("../assets/icons/ellipsis.svg"),
            Self::ExternalLink => include_bytes!("../assets/icons/external-link.svg"),
            Self::CircleCheck => include_bytes!("../assets/icons/circle-check.svg"),
            Self::CircleX => include_bytes!("../assets/icons/circle-x.svg"),
            Self::Bell => include_bytes!("../assets/icons/bell.svg"),
        }
    }

    #[must_use]
    pub fn from_asset_path(path: &str) -> Option<Self> {
        let stem = path.strip_prefix(PREFIX)?.strip_suffix(".svg")?;
        Self::ALL.into_iter().find(|icon| icon.stem() == stem)
    }
}

impl IconNamed for IconName {
    fn path(self) -> SharedString {
        self.asset_path()
    }
}

/// An icon element sized from the shared scale and tinted with a token colour
/// (the current text colour by default).
#[derive(IntoElement, Debug, Clone, Copy)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
}

impl Icon {
    #[must_use]
    pub const fn new(name: IconName) -> Self {
        Self {
            name,
            size: size::ICON_MD,
            color: None,
        }
    }

    #[must_use]
    pub const fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub const fn small(self) -> Self {
        self.size(size::ICON_SM)
    }

    #[must_use]
    pub const fn large(self) -> Self {
        self.size(size::ICON_LG)
    }

    #[must_use]
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let icon = ToolkitIcon::new(self.name).with_size(self.size);
        match self.color {
            Some(color) => icon.text_color(color),
            None => icon,
        }
    }
}

/// Serves the shell's icons and delegates every other path to the toolkit's
/// bundled assets.
#[derive(Debug, Clone, Copy, Default)]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        IconName::from_asset_path(path).map_or_else(
            || gpui_kit::assets::Assets.load(path),
            |icon| Ok(Some(Cow::Borrowed(icon.svg()))),
        )
    }

    fn list(&self, path: &str) -> gpui_kit::Result<Vec<SharedString>> {
        let mut paths: Vec<SharedString> = IconName::ALL
            .into_iter()
            .map(IconName::asset_path)
            .filter(|asset| asset.starts_with(path))
            .collect();
        paths.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(paths)
    }
}
