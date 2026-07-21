//! Read paths for user data, split by the page that consumes them.

mod detail;
mod events;
mod listing;
mod role;
mod runtime;

pub use detail::{
    find_user_detail, get_user_event_type_breakdown, get_user_sessions, get_user_top_tools,
};
pub use events::get_user_usage;
pub use listing::{fetch_distinct_roles, list_users};
pub use role::get_user_roles_department;
pub use runtime::{
    UserRuntimeAggregate, UserRuntimeDetail, get_user_runtime_detail, list_user_runtime_aggregates,
};
