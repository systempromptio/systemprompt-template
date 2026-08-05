//! Integration coverage for `systemprompt-web-admin`'s repositories against a
//! live Postgres: the configured-policy surface (`config`), the marketplace's
//! catalog, usage and environment records, encrypted secret storage, and the
//! scheduled-job list.
//!
//! Every test runs against its OWN throwaway database created on the server
//! named by `DATABASE_URL`, with the real extension schema installed, so the
//! shared application tables are never read, written, or truncated. The
//! database is dropped on completion, and each test self-skips when no test
//! database URL is configured.
//!
//! That schema is not empty: the web extension's migrations create a default
//! department and seed dashboard rows. Assertions therefore name the ids they
//! seeded rather than counting rows table-wide.

#[cfg(test)]
mod config_acl_detect;
#[cfg(test)]
mod config_gateway_acl;
#[cfg(test)]
mod config_roles;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod jobs_repo;
#[cfg(test)]
mod marketplace_catalog;
#[cfg(test)]
mod marketplace_env;
#[cfg(test)]
mod marketplace_usage;
#[cfg(test)]
mod secrets_keys;
#[cfg(test)]
mod secrets_migration;
#[cfg(test)]
mod secrets_resolve;
#[cfg(test)]
mod tempdb;
