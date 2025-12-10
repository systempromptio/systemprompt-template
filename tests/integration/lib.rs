pub mod agents;
/// Integration tests root module
///
/// Organized by domain:
/// - ai: AI request lifecycle, tool calls, usage analytics, SQLX migrations
/// - analytics: Session creation, events, endpoints, AI usage, UTM, GeoIP,
///   integrity
/// - agents: A2A protocol, conversation, tasks, messages, tools, streaming
/// - auth: OAuth flow, JWT validation, session management, permissions
/// - content: Blog, static pages, ingestion, rendering
/// - mcp: MCP server lifecycle, tools, resources, prompts
/// - database: Foreign keys, constraints, migrations, orphaned records
pub mod ai;
pub mod analytics;
pub mod auth;
pub mod common;
pub mod content;
pub mod database;
pub mod mcp;
