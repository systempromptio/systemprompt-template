//! Gateway route configuration backed by the profile YAML.
//!
//! The gateway config is not a Postgres table: it lives in the profile YAML's
//! `gateway` block, which is why it sits here. These functions read, mutate,
//! and re-serialize that block. Reads never write, and a write touches the
//! edited field only: the file is an operator's, comments and all.

mod catalog;
mod config;
mod matching;
mod routes;
mod yaml_io;

pub use catalog::{
    client_facing_routes, client_facing_routes_from_services, dispatchable_route_ids,
    dispatchable_routes, dispatchable_routes_from_services, registered_routes,
    registered_routes_from_services, retain_client_facing,
};
pub use config::{get_gateway_config, update_gateway_settings};
pub use matching::{
    find_matching_route, find_matching_route_index, find_route_index_by_id, glob_match,
    slugify_pattern, synthesize_route_id,
};
pub use routes::{create_route, delete_route, reorder_routes, update_route, validate_route};
