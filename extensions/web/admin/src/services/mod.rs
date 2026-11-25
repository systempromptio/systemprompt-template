//! Service layer between the admin handlers and the repositories.

#[cfg(feature = "agents")]
pub(crate) mod analysis_report;
pub(crate) mod auth;
pub(crate) mod bridge_profile;
pub(crate) mod device_service;
pub(crate) mod jobs_service;
pub(crate) mod marketplaces;
pub(crate) mod secret_service;

pub mod connector_oauth;

pub(crate) mod connector_accounts;
#[cfg(feature = "mcp")]
pub(crate) mod connector_probe;
pub(crate) mod connector_readiness;
pub mod identity_token;
