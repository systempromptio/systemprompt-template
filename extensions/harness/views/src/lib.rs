//! The window's views: the frame and its navigation, and one module per screen.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

pub mod composer;
pub mod docks;
pub(crate) mod extension_entries;
pub(crate) mod fatal;
pub(crate) mod frame;
pub mod governance;
pub(crate) mod home;
pub mod pickers;
pub(crate) mod projects;
pub(crate) mod session_menu;
pub mod settings;
pub(crate) mod sidebar;
pub mod signin;
pub(crate) mod titlebar;
pub(crate) mod toasts;
pub mod transcript;
pub(crate) mod window_state;
