//! Platform-organization membership: the argument surface and the subcommand
//! bodies behind `systemprompt plugins run platform`.
//!
//! "Platform admin" is the `admin` role **plus** membership in the single
//! `is_platform` organization. Core's role commands cover the first half;
//! this extension covers the second, which previously had no CLI at all —
//! the only write paths were the Admin UI (itself gated on platform admin)
//! and raw SQL.
//!
//! The queries live in `systemprompt_web_admin::repositories::organizations`,
//! next to the membership code the Admin UI uses, so the CLI and the UI cannot
//! drift. This crate is the parser and the printing around them.

// Why: stdout is this crate's entire interface — it backs a CLI, and the
// workspace lints deny printing by default because most crates here are
// libraries.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI binary: stdout and stderr are the user-facing output"
)]

pub mod cli;
pub mod commands;
