mod audit;
mod authz;
mod handler;
mod policies;
pub mod policy;
mod scope;
mod secrets;
mod types;

pub use authz::govern_authz;
pub use handler::govern_tool_use;
pub use policy::{chain, reload};

/// Reserved key in [`systemprompt_security::policy::McpToolInput`] used by
/// the template handler to plumb the OAuth scope label
/// ("admin" / "user" / "unknown") to policies that gate on admin status.
/// Lives on the boundary wrapper because core's [`PolicyContext`] is
/// deliberately deployment-agnostic.
pub(crate) const SCOPE_LABEL_KEY: &str = "__systemprompt_scope_label";
