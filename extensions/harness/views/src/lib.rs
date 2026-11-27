//! The window's views: the frame and its navigation, and one module per
//! screen, over the UI model ([`state`]), the services they are built on
//! ([`app`]), the keyboard commands and the strings bundle.
//!
//! [`brain`], [`drive`] and [`notifications`] are shell extensions, named in
//! [`registered::REGISTERED`].

pub mod app;
pub mod brain;
pub mod composer;
pub mod docks;
pub mod drive;
pub(crate) mod extension_entries;
pub mod fatal;
pub mod frame;
pub mod governance;
pub(crate) mod home;
pub mod keymap;
pub mod notifications;
pub mod pickers;
pub(crate) mod projects;
pub mod registered;
pub(crate) mod session_menu;
pub mod settings;
pub(crate) mod sidebar;
pub mod signin;
pub mod state;
pub mod strings;
pub(crate) mod titlebar;
pub(crate) mod toasts;
pub mod transcript;
pub mod window_state;
