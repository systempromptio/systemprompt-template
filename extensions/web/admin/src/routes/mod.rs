//! Router construction for the admin plane.

mod admin;
mod ssr;
mod ssr_redirects;

pub(crate) use admin::{build_admin_only_routes, build_auth_read_routes};
pub use ssr::admin_ssr_router;

mod admin_groups;
pub(crate) use admin::build_self_service_routes;

mod ssr_bridge;
pub use ssr_bridge::bridge_auth_ssr_router;

mod ssr_write_gate;

mod dashboard_redirects;

mod ssr_dashboard;
