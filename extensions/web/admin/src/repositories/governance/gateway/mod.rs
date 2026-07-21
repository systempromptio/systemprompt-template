//! Gateway route configuration backed by the profile YAML.
//!
//! Unlike the rest of `governance`, the gateway config is not a Postgres
//! table: it lives in the profile YAML's `gateway` block. These functions read,
//! mutate, and re-serialize that block while keeping every route's stable `id`
//! synchronized.

mod config;
mod matching;
mod routes;
mod yaml_io;

pub use config::{get_gateway_config, update_gateway_settings};
pub use matching::{
    find_matching_route, find_matching_route_index, find_route_index_by_id, glob_match,
    slugify_pattern, synthesize_route_id,
};
pub use routes::{
    create_route, delete_route, ensure_route_ids, reorder_routes, update_route, validate_route,
};
