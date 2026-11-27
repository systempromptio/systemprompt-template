//! The shell's design system (components, markdown, diffs, code blocks, icons
//! and motion over the theme crate's tokens) and the shell extension
//! [`contract`].
//!
//! The design system builds on the toolkit (`gpui-kit`) and knows nothing
//! about the engine, the store or an instance; only [`contract`] names core's
//! crates. Screens use these pieces instead of styling raw elements, so the
//! app looks and behaves the same everywhere.
//!
//! Start-up, in this order:
//!
//! ```ignore
//! gpui_kit::application()
//!     .with_assets(systemprompt_harness_shared::Assets)
//!     .run(|cx| {
//!         systemprompt_harness_shared::init(cx);
//!         // open windows with gpui_kit::open_window(..)
//!     });
//! ```

pub mod code;
pub mod components;
pub mod contract;
pub mod diff;
pub mod icons;
pub mod markdown;
pub mod motion;

use gpui_kit::App;
pub use systemprompt_harness_theme as theme;

pub use code::CodeBlock;
pub use components::tooltip;
pub use diff::{DiffStat, DiffView, Patch};
pub use icons::{Assets, Icon, IconName};
pub use markdown::{Markdown, MarkdownDocument};
pub use theme::{BrandTheme, Palette, Scheme, Tone, tokens};

pub fn init(cx: &mut App) {
    gpui_kit::init(cx);
    theme::apply(cx, BrandTheme::current(), Scheme::System);
    components::bind_keys(cx);
}
