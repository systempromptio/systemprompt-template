//! The shell's extension contract.
//!
//! A feature with its own screens is a [`ShellExtension`]: a
//! `Default`-constructed type registered with [`register_shell_extension!`],
//! keyed by the same extension id as its instance-side half, and found at
//! startup by [`ShellExtensions::discover`]. It contributes screens placed in
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
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod actions;
mod context;
mod contribution;
mod discovery;
mod extension;
mod register;

#[doc(hidden)]
pub use inventory as __inventory;

pub use actions::DraftInSession;
pub use context::ShellContext;
pub use contribution::{
    OnChange, OpenScreen, Placement, ScreenKey, ScreenSpec, SettingsPaneSpec, TitlebarItemBuilder,
    TitlebarItemSpec, TranscriptAdornment, ViewBuilder,
};
pub use discovery::ShellExtensions;
pub use extension::ShellExtension;
pub use harness_engine::extension::{Contributed, ExtensionMetadata};
pub use harness_ids::{ExtensionId, extension_id};
pub use register::ShellExtensionRegistration;
