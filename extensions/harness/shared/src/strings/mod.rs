//! Every user-facing string of the shell, in English, one table module per
//! area.
//!
//! A screen reads its text from its
//! own table and from `foundation` for shared words; nothing user-facing is
//! written inline in a view.

pub(crate) mod composer;
pub(crate) mod docks;
pub(crate) mod foundation;
pub(crate) mod governance;
pub(crate) mod pickers;
pub(crate) mod settings;
pub mod signin;
pub(crate) mod transcript;
