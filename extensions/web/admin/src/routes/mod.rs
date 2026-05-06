mod admin;
mod ssr;
mod ssr_cowork;

pub use admin::{build_admin_only_routes, build_auth_read_routes};
pub use ssr::admin_ssr_router;
pub use ssr_cowork::cowork_auth_ssr_router;
