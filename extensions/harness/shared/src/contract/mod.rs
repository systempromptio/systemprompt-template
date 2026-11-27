//! The shell's extension contract.
//!
//! A feature with its own screens is a [`ShellExtension`]: a
//! `Default`-constructed type named in a registration list of
//! [`ShellExtensionRegistration`]s, keyed by the same extension id as its
//! instance-side half, and built at startup by
//! [`ShellExtensions::from_registrations`]. It contributes screens placed in
//! the sidebar or the titlebar (optionally with a titlebar item it draws
//! itself, such as a live count), settings panes and transcript adornments
//! that may ask to be drawn again, each built from the [`ShellContext`] the
//! composition root made. What it shows
//! follows the instance's enablement exactly as its instance-side half does: a
//! withheld extension shows nothing unless it is required. A view asks the
//! frame for what only the frame can do (such as [`DraftInSession`]) by
//! dispatching an action from its window.
//!
//! Only this ring depends on the UI toolkit.

mod actions;
mod context;
mod contribution;
mod discovery;
mod extension;
mod register;

pub use actions::DraftInSession;
pub use context::ShellContext;
pub use contribution::{
    OnChange, OpenScreen, Placement, ScreenKey, ScreenSpec, SettingsPaneSpec, TitlebarItemBuilder,
    TitlebarItemSpec, TranscriptAdornment, ViewBuilder,
};
pub use discovery::ShellExtensions;
pub use extension::ShellExtension;
pub use register::ShellExtensionRegistration;
pub use systemprompt_engine::run::extension::{Contributed, ExtensionMetadata};
pub use systemprompt_identifiers::engine::ExtensionId;
pub use systemprompt_identifiers::extension_id;
