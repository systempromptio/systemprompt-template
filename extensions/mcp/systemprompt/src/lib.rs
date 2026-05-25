//! MCP server crate for the Enterprise Demo template.
//!
//! Implements the `systemprompt` MCP server that ships with the demo plugin.
//! Tools are defined in [`tools`] and exposed through [`SystempromptServer`];
//! errors normalise on [`error::McpError`]. The `main` binary is a thin
//! `tokio::main` shell that builds a [`SystempromptServer`] and serves it
//! over stdio.

mod cli;
pub mod error;
pub mod server;
pub mod tools;

pub use server::SystempromptServer;
