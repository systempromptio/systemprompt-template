//! Persistence for the admin dashboard and control centre.

pub mod aggregates;
pub mod apm_metrics;
pub mod control_center;
pub mod conversation_analytics;
pub mod hooks_track;
pub mod overview;
pub mod queries;
pub mod session_analyses;
pub mod traffic;
pub mod usage_aggregations;

pub use overview::{get_dashboard_data, list_events};
