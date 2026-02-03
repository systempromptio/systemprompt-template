//! `SystemPrompt` Template
//!
//! This crate re-exports extensions for use with the `SystemPrompt` runtime.
//! Extensions are automatically discovered via the `inventory` crate.

pub use systemprompt::cli;
pub use systemprompt::*;
pub use systemprompt_soul_extension as soul;
pub use systemprompt_web_extension as web;
