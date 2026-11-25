//! MCP server crate for the systemprompt template.
//!
//! Implements the `systemprompt` MCP toolset that ships with the demo plugin.
//! Tools are defined in [`tools`] and served by [`SystempromptToolset`], which
//! the [`SystempromptMcpExtension`] member contributes to core's toolset host;
//! the composed binary hosts it as `admin mcp serve --server systemprompt`.
//! Errors normalise on [`error::SystempromptToolError`].

#[doc(hidden)]
pub mod bounds;
mod cli;
pub mod error;
mod extension;
#[doc(hidden)]
pub mod reports;
#[doc(hidden)]
pub mod typed;

#[doc(hidden)]
pub use cli::{CliError, CliLocation, filter_hallucinated_args};
pub mod server;
pub mod tools;

pub use extension::SystempromptMcpExtension;
pub use server::SystempromptToolset;
