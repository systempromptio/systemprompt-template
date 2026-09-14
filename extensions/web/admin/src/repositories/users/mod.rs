//! Persistence for users: identity, access, activity, devices, and usage.

pub mod access_control;
pub mod activity;
pub mod aggregates;
pub mod devices;
pub mod enrolment;
pub mod federated;
mod federated_provision;
pub mod mutations;
pub mod queries;
pub mod revocation;
pub mod roles;
pub mod roster;
pub mod salesforce_identity;
pub mod sessions;
pub mod share_token;
pub mod usage;
pub mod user_settings;

pub use mutations::{create_user, delete_user, update_user};
pub use share_token::find_share_token_version;

pub mod connector_credentials;

pub mod connector_accounts;
