mod admin;
mod ssr;
mod ssr_bridge;

pub(crate) use admin::{build_admin_only_routes, build_auth_read_routes};
pub use ssr::admin_ssr_router;
pub use ssr_bridge::bridge_auth_ssr_router;
