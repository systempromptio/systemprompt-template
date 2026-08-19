//! Integration coverage for the pool-backed halves of the web and MCP
//! extensions against a live Postgres: `systemprompt-web-content`'s
//! repositories (content CRUD and orphan pruning, campaign-link lookup, click
//! tracking and its counters, content search) and the service layer over them,
//! markdown ingestion from a real directory tree, the content-analytics job's
//! rollups, and construction of the bundled MCP server.
//!
//! Every test runs against its OWN throwaway database created on the server
//! named by `DATABASE_URL`, with the real extension schema installed, so the
//! shared application tables are never read, written, or truncated. The
//! database is dropped on completion, and each test self-skips when no test
//! database URL is configured.

#[cfg(test)]
mod content_api;
#[cfg(test)]
mod content_ingestion;
#[cfg(test)]
mod content_repository;
#[cfg(test)]
mod content_services;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod jobs_context;
#[cfg(test)]
mod jobs_db;
#[cfg(test)]
mod link_analytics_repository;
#[cfg(test)]
mod link_repository;
#[cfg(test)]
mod mcp_cli;
#[cfg(test)]
mod mcp_dispatch;
#[cfg(test)]
mod mcp_server;
#[cfg(test)]
mod search_repository;
#[cfg(test)]
mod site_docs_db;
#[cfg(test)]
mod tempdb;
#[cfg(test)]
mod web_router;
