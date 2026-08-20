//! Customer organizations: the tenancy unit of the hosted offering.
//!
//! [`crud`] owns the organization and membership rows. [`seats`] owns the one
//! rule that turns a contract into an enforceable limit — a seat is an active
//! member, and the limit is checked at every point that mints one.
//!
//! Entitlement is deliberately absent from this module. What an organization
//! may reach lives in `access_control_rules` at `rule_type = 'organization'`,
//! written by [`super::config::plan_yaml_loader`] and read by the ordinary
//! resolver, so there is no second authorization path to keep in sync.
//!
//! [`metrics`] reads the numbers the enterprise console leads with — seats,
//! footprint, cost, and margin — across every customer at once; [`detail`]
//! reads what one of them is made of. [`spend`] owns the month-to-date query
//! the gateway budget guard also runs, so what an operator is shown and what a
//! customer is throttled on cannot drift apart.

pub mod budget_warnings;
pub mod crud;
pub mod detail;
pub mod metrics;
pub mod seats;
pub mod spend;
