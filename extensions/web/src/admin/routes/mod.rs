mod admin;
mod ssr;
mod user;

pub(super) use admin::{build_admin_only_routes, build_auth_read_routes};
pub(crate) use ssr::{admin_ssr_router, workspace_ssr_router};
pub(super) use user::build_auth_write_routes;
