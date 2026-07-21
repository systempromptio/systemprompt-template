//! Data access for the admin surface, one module per domain.
//!
//! Callers path-qualify (`repositories::governance::gateway::create_route`);
//! this module re-exports nothing, so the module path is the only name a
//! symbol has and collisions between domains cannot arise.

pub mod analytics;
pub mod bridge;
pub mod dashboard;
pub mod departments;
pub mod external_agents;
pub mod governance;
pub mod marketplace;
pub mod mcp;
pub mod secrets;
pub mod traces;
pub mod users;
