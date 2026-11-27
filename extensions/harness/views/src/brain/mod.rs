//! The Brain's shell half.
//!
//! The Brain's screens, settings pane and transcript adornment, as the shell
//! half of extension `brain`. [`BrainShellExtension`] is registered with
//! `register_shell_extension!` and contributes, while the instance admits the
//! extension, a sidebar screen (the facts with their filters, paging, detail,
//! timeline and why-view, the person's own actions on a fact, and the proposals
//! inbox), a settings pane with the recall switch, and a transcript adornment
//! naming the facts recalled into a session's latest model call. Every view
//! reads the [`BrainService`](sp_brain::BrainService) the instance-side half
//! provides and follows its feed; nothing is shown as usable unless the
//! instance admits the extension and has confirmed its brain is enabled.
//!
//! - [`model`]: the views' pure state, tested on its own.
//! - [`strings`]: every user-facing string of this crate.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod extension;
pub mod model;
pub mod strings;
mod view;

pub use extension::BrainShellExtension;
