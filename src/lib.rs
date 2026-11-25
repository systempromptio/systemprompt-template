//! systemprompt.io evaluation template.
//!
//! This crate re-exports the template's members and builds the composition the
//! binary runs: every core member the selected flavour links, plus the
//! template's own members.

#[cfg(not(feature = "flavour-full"))]
compile_error!(
    "no flavour selected: build with --no-default-features --features flavour-<name> (just flavours lists them)"
);

pub use systemprompt_mcp_agent as mcp;
pub use systemprompt_web_extension as web;

#[must_use]
pub fn composition() -> systemprompt::Composition {
    systemprompt::reference()
        .with(web::WebExtension)
        .with(mcp::SystempromptMcpExtension)
}
