//! Reads for the roles and permissions page.
//!
//! A role is not a row anywhere: it is a string in `users.roles`, a grant in
//! `user_manual_roles`, and a subject band in `access_control_rules`. The two
//! modules here read those three tables from the role's side rather than the
//! user's — who holds it, and what holding it opens — which is the question
//! the page exists to answer and the one no per-user query can answer without
//! being run once per account.

pub mod entitlements;
pub mod members;
