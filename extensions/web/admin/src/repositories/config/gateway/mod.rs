//! Gateway route configuration backed by the profile YAML.
//!
//! The gateway config is not a Postgres table: it lives in the profile YAML's
//! `gateway` block, which is why it sits here. These functions read, mutate,
//! and re-serialize that block. Reads never write, and a write touches the
//! edited field only: the file is an operator's, comments and all.

mod catalog;
mod config;
mod labels;
mod matching;
mod routes;
mod yaml_io;

pub use catalog::{
    client_facing_routes, client_facing_routes_from_services, dispatchable_route_ids,
    dispatchable_routes, dispatchable_routes_from_services, registered_routes,
    registered_routes_from_services, retain_client_facing,
};
pub use config::{get_gateway_config, update_gateway_settings};
pub use labels::{
    ProviderLabel, RouteLabel, RouteLabels, derive_provider_label, derive_route_label,
    get_route_labels, get_route_labels_from_services,
};
pub use matching::{find_matching_route, glob_match, slugify_pattern, synthesize_route_id};
pub use routes::{
    create_route, delete_route, normalise_metadata, reorder_routes, replace_routes, update_route,
    validate_route,
};
pub use yaml_io::{read_gateway_file, route_from_yaml, route_to_yaml};
