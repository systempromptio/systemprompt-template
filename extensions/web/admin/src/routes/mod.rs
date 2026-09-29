//! Router construction for the admin plane.

mod admin;
mod admin_groups;
mod managed_resources;
pub(crate) mod managed_state;
mod ssr;
mod ssr_analysis;
mod ssr_bridge;
mod ssr_export;
mod ssr_governance;
mod ssr_platform;
mod ssr_redirects;

pub(crate) use admin::{
    build_admin_only_routes, build_auth_read_routes, build_self_service_routes,
};
pub use ssr::admin_ssr_router;
pub use ssr_bridge::bridge_auth_ssr_router;
