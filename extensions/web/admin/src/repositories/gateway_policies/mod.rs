//! The gateway policy plane: `services/gateway/policies.yaml` projected into
//! `ai_gateway_policies`.
//!
//! Core re-reads `ai_gateway_policies` on every inference request (a
//! sixty-second cache), so a row written here is live without a restart.
//! The declaration stays the file: [`declared`] reads it, [`drift`] compares
//! it with the rows, [`export`] renders the rows back as the file, and
//! [`sync::gateway_policies`](crate::repositories::sync::gateway_policies)
//! is the plane the sync page drives.
//!
//! Calendar-month quota windows are an extension-side interim over core's
//! fixed-length windows: [`month_window`] holds the arithmetic and
//! [`month_window_db`] the daily rewrite. Read that module head before
//! touching a window longer than 31 days.

pub mod declared;
pub mod drift;
pub mod export;
pub mod month_window;
pub mod month_window_db;
pub mod rows;
