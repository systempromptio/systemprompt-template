//! The services under the views.
//!
//! Paths, services and their startup, the systemprompt side and the linked
//! extensions, preferences, the tokio bridge and links. The shell crate adds
//! the single instance, logging and the main window's lifecycle.

pub(crate) mod browser;
pub mod deeplink;
pub mod extension_services;
pub(crate) mod governed;
pub mod governor;
pub(crate) mod linked;
pub mod paths;
pub(crate) mod runtime;
pub(crate) mod services;
pub mod settings;
pub mod settings_store;
pub(crate) mod startup;

pub use services::{Services, ServicesConfig};
