//! Persistence for the admin dashboard and control centre.

#[cfg(feature = "mcp")]
pub mod aggregates;
pub mod apm_metrics;
pub mod commits;
pub mod conversation_analytics;
pub mod hooks_track;
pub mod overview;
#[cfg(feature = "mcp")]
pub mod queries;
#[cfg(feature = "agents")]
pub mod session_analyses;
#[cfg(feature = "content")]
pub mod traffic;
pub mod usage_aggregations;
pub mod usage_rollups;

pub use overview::{DashboardWindows, get_dashboard_data, list_events};
