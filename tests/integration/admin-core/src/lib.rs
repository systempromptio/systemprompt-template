//! Integration coverage for `systemprompt-web-admin`'s repositories against a
//! live Postgres: user identity and provisioning, organizations and seats,
//! departments, the analytics read models, the dashboard counters, the
//! governance record, and the trace explorer.
//!
//! Every test runs against its OWN throwaway database created on the server
//! named by `DATABASE_URL`, with the real extension schema installed, so the
//! shared application tables are never read, written, or truncated. The
//! database is dropped on completion, and each test self-skips when no test
//! database URL is configured.
//!
//! Installing the real schema also runs the web extension's migrations, and
//! several of those seed rows — the `house` organization, the `Default`
//! department, three demo customers and their inference history. The suite
//! therefore asserts on rows it inserted and on deltas rather than on a table
//! being empty; `fixtures` documents the baseline in detail.


#[cfg(test)]
mod analytics_content_metrics;
#[cfg(test)]
mod analytics_content_rollup;
#[cfg(test)]
mod analytics_context_detail;
#[cfg(test)]
mod analytics_context_messages;
#[cfg(test)]
mod analytics_contexts;
#[cfg(test)]
mod analytics_request_stats;
#[cfg(test)]
mod analytics_requests;
#[cfg(test)]
mod analytics_session_children;
#[cfg(test)]
mod analytics_session_detail;
#[cfg(test)]
mod analytics_site;
#[cfg(test)]
mod dashboard_apm;
#[cfg(test)]
mod dashboard_counters;
#[cfg(test)]
mod dashboard_entity_links;
#[cfg(test)]
mod dashboard_session_analyses;
#[cfg(test)]
mod dashboard_session_summary;
#[cfg(test)]
mod dashboard_session_updates;
#[cfg(test)]
mod dashboard_sessions;
#[cfg(test)]
mod dashboard_traffic_content;
#[cfg(test)]
mod dashboard_traffic_geo;
#[cfg(test)]
mod dashboard_traffic_queries;
#[cfg(test)]
mod dashboard_usage_daily;
#[cfg(test)]
mod departments_repository;
#[cfg(test)]
mod departments_rollups;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod governance_counts;
#[cfg(test)]
mod governance_facets;
#[cfg(test)]
mod invites_regenerate;
#[cfg(test)]
mod mcp_servers_yaml;
#[cfg(test)]
mod organizations_membership;
#[cfg(test)]
mod organizations_seats;
#[cfg(test)]
mod req_026_audit_completeness;
#[cfg(test)]
mod tempdb;
#[cfg(test)]
mod traces_list;
#[cfg(test)]
mod traces_spans;
#[cfg(test)]
mod traces_spans_resolve;
#[cfg(test)]
mod traces_stats;
#[cfg(test)]
mod usage_metrics;
#[cfg(test)]
mod users_access_matrix;
#[cfg(test)]
mod users_access_matrix_dimensions;
#[cfg(test)]
mod users_access_rules;
#[cfg(test)]
mod users_activity_record;
#[cfg(test)]
mod users_federated_sso;
#[cfg(test)]
mod users_identity;
#[cfg(test)]
mod users_lookups;
#[cfg(test)]
mod users_passkey;
