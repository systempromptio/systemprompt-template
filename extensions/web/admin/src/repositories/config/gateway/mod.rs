//! Gateway route configuration backed by the services tree.
//!
//! The gateway config is not a Postgres table: core 0.44 moved it out of the
//! profile into `services/ai/gateway.yaml`, which is why it sits here. Reads
//! come from the loaded services tree so the admin surface reports what the
//! gateway dispatches by; writes edit that file, never the profile.

mod catalog;
mod config;
mod matching;
mod path;
mod routes;
mod yaml_io;

pub use catalog::{dispatchable_route_ids, registered_routes, registered_routes_from_services};
pub use config::{get_gateway_config, get_gateway_config_from_file, update_gateway_settings};
pub use matching::{
    find_matching_route, find_matching_route_index, find_route_index_by_id, glob_match,
    slugify_pattern, synthesize_route_id,
};
pub use path::gateway_config_path;
pub use routes::{
    create_route, delete_route, ensure_route_ids, reorder_routes, update_route, validate_route,
};
