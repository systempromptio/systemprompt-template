//! systemprompt.io evaluation template.
//!
//! This crate re-exports the template's members and names the composition the
//! binary runs: the reference composition with the web extension and the
//! `systemprompt` MCP toolset added with install standing.

pub use systemprompt_mcp_agent as mcp;
pub use systemprompt_web_extension as web;

#[must_use]
pub fn composition() -> systemprompt::Composition {
    systemprompt::reference()
        .with(web::WebExtension)
        .with(mcp::SystempromptMcpExtension)
}
