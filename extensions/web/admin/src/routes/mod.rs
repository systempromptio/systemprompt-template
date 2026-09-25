//! Router construction for the admin plane.

mod admin;
mod admin_groups;
mod ssr;
mod ssr_bridge;
mod ssr_redirects;

pub(crate) use admin::{
    build_admin_only_routes, build_auth_read_routes, build_self_service_routes,
};
pub use ssr::admin_ssr_router;
pub use ssr_bridge::bridge_auth_ssr_router;
