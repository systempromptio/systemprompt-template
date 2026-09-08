//! Service layer between the admin handlers and the repositories.

pub(crate) mod access_token_service;
pub(crate) mod auth;
pub(crate) mod bridge_profile;
pub(crate) mod connector_accounts;
pub mod connector_oauth;
pub(crate) mod device_service;
pub(crate) mod evals;
pub(crate) mod jobs_service;
pub(crate) mod marketplaces;
pub(crate) mod salesforce_jwt_bearer;
pub(crate) mod secret_service;

pub(crate) mod bridge_downloads;
