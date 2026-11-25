//! systemprompt.io evaluation template.
//!
//! This crate re-exports the template's members and builds the composition the
//! binary runs: every core member the selected flavour links, plus the
//! template's own members that flavour compiles.

#[cfg(not(any(
    feature = "flavour-gateway",
    feature = "flavour-mcp",
    feature = "flavour-agents",
    feature = "flavour-site",
    feature = "flavour-full"
)))]
compile_error!(
    "no flavour selected: build with --no-default-features --features flavour-<name> (just flavours lists them)"
);

#[cfg(feature = "flavour-mcp")]
pub use systemprompt_mcp_agent as mcp;
pub use systemprompt_web_extension as web;

#[must_use]
pub fn composition() -> systemprompt::Composition {
    let composition = systemprompt::reference().with(web::WebExtension);
    #[cfg(feature = "flavour-mcp")]
    let composition = composition.with(mcp::SystempromptMcpExtension);
    composition
}
