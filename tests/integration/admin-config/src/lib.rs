//! Integration coverage for `systemprompt-web-admin`'s repositories against a
//! live Postgres: the configured-policy surface (`config`), the marketplace's
//! usage and environment records, the two month-end report families, encrypted
//! secret storage, the bridge control plane, and the scheduled-job list.
//!
//! Every test runs against its OWN throwaway database created on the server
//! named by `DATABASE_URL`, with the real extension schema installed, so the
//! shared application tables are never read, written, or truncated. The
//! database is dropped on completion, and each test self-skips when no test
//! database URL is configured.
//!
//! That schema is not empty: the web extension's migrations seed a house
//! organization and three demo tenants with thirty days of traffic. Assertions
//! therefore name the ids they seeded, or read a month in 2001 that no seeded
//! row can reach.

#[cfg(test)]
mod bridge_api_keys;
#[cfg(test)]
mod bridge_identity;
#[cfg(test)]
mod config_acl_detect;
#[cfg(test)]
mod config_gateway_acl;
#[cfg(test)]
mod config_plans;
#[cfg(test)]
mod config_roles;
#[cfg(test)]
mod config_salesforce;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod jobs_repo;
#[cfg(test)]
mod marketplace_catalog;
#[cfg(test)]
mod marketplace_env;
#[cfg(test)]
mod marketplace_filter;
#[cfg(test)]
mod marketplace_usage;
#[cfg(test)]
mod reports_customer_lists;
#[cfg(test)]
mod reports_customer_summary;
#[cfg(test)]
mod reports_pnl;
#[cfg(test)]
mod reports_suppliers;
#[cfg(test)]
mod secrets_keys;
#[cfg(test)]
mod secrets_migration;
#[cfg(test)]
mod secrets_resolve;
#[cfg(test)]
mod tempdb;
