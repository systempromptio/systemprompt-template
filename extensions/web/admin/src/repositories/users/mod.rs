//! Persistence for users: identity, access, activity, access tokens, and usage.

pub mod access_control;
pub mod access_tokens;
pub mod access_tree;
pub mod activity;
pub mod federated;
pub mod identity;
pub mod magic_links;
pub mod mutations;
pub mod queries;
pub mod registration;
pub mod share_token;
pub mod usage;
pub mod user_queries;
pub mod user_settings;

pub use mutations::{create_user, delete_user, update_user};
pub use share_token::find_share_token_version;
