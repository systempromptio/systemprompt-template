//! The shell's own plumbing: single instance, logging and the main window's
//! lifecycle. Paths, services and preferences are the views crate's `app`.

pub(crate) mod headless;
pub(crate) mod logging;
pub mod single_instance;
pub(crate) mod tls;
pub(crate) mod window;
