//! Config-level evidence for the gateway requirements in the enterprise
//! requirements register that are not provable through the browser.
//!
//! Each `req_0nn_*` module opens with the register acceptance criterion it
//! evidences and exercises the real profile types (`GatewayConfig`,
//! `GatewayRoute`, `RouteRequirements`, `ProviderRegistry`,
//! `ModelGovernance`) — the same structs the production loader deserializes
//! from `profile.yaml` — so a passing test is proof about the shipped
//! enforcement code, not about a re-implementation. No database is needed:
//! routing, residency validation, and exposure posture are all decided from
//! configuration alone.

#[cfg(test)]
mod req_020_provider_abstraction;
#[cfg(test)]
mod req_024_shadow_ai;
#[cfg(test)]
mod req_033_quota_config;
#[cfg(test)]
mod req_037_residency;
#[cfg(test)]
mod req_038_no_retain;
#[cfg(test)]
mod support;
