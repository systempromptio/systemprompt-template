//! Read paths for user data, split by the page that consumes them.

mod detail;
mod events;
mod identity;
mod listing;
mod role;
mod runtime;

pub use detail::{
    find_user_detail, list_user_event_type_breakdown, list_user_sessions, list_user_top_tools,
};
pub use events::list_user_usage;
pub use identity::{IdentityEnvelope, find_identity_envelope};
pub use listing::{list_distinct_roles, list_users};
pub use role::{UserAccessProfile, find_user_access_profile};
pub use runtime::{
    UserRuntimeAggregate, UserRuntimeDetail, get_user_runtime_detail, list_user_runtime_aggregates,
};

pub use role::find_user_roles_department;
