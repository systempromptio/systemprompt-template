mod admin;
mod user;

pub(super) use admin::{build_admin_only_routes, build_auth_read_routes};
pub(super) use user::build_auth_write_routes;
