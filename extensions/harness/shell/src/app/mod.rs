//! The shell's plumbing.
//!
//! Paths, services and their startup, the systemprompt side and the linked
//! extensions, preferences, the tokio bridge, links, single instance, logging
//! and the main window's lifecycle.

pub(crate) mod browser;
pub mod deeplink;
pub mod extension_services;
pub(crate) mod governed;
pub mod governor;
pub(crate) mod linked;
pub(crate) mod logging;
pub mod paths;
pub(crate) mod runtime;
pub(crate) mod services;
pub mod settings;
pub mod settings_store;
pub mod single_instance;
pub(crate) mod startup;
pub(crate) mod window;

pub(crate) use services::{Services, ServicesConfig};
