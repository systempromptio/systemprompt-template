//! Tool handlers, authentication, and dispatch for the knowledge-bank MCP
//! server.
//!
//! The server in the parent module owns the rmcp `ServerHandler` surface;
//! this module owns what happens per tool call: RBAC enforcement against the
//! registry, access auditing, and turning retrieval results into a
//! [`CliArtifact`]. A `NotConfigured` backend answer is surfaced as an honest
//! text artifact rather than a protocol error, so a client sees exactly why
//! it got no results.

use crate::error::KnowledgeBankError;
use crate::retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter,
};
use crate::tools::{
    INDEX_STATS_TOOL, IndexStatsInput, LIST_SOURCES_TOOL, ListKnowledgeSourcesInput, SEARCH_TOOL,
    SearchKnowledgeInput,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::{RequestContext, RoleServer};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

fn not_configured_artifact(reason: &str) -> (CliArtifact, String) {
    let summary = format!("Knowledge backend not configured: {reason}");
    (
        CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Bank")),
        summary,
    )
}

fn format_hits(hits: &[KnowledgeHit], query: &str) -> (CliArtifact, String) {
    let body = hits
        .iter()
        .map(|h| {
            format!(
                "[{:.2}] {} ({}/{})\n{}\n{}",
                h.score, h.title, h.source_type, h.source, h.snippet, h.uri
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Search Results")),
        format!("{} result(s) for `{query}`", hits.len()),
    )
}

fn format_sources(sources: &[KnowledgeSource]) -> (CliArtifact, String) {
    if sources.is_empty() {
        let summary = "No knowledge sources are configured on this instance.".to_owned();
        return (
            CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Sources")),
            summary,
        );
    }
    let body = sources
        .iter()
        .map(|s| {
            let docs = s
                .doc_count
                .map_or_else(String::new, |n| format!(", {n} docs"));
            let synced = s
                .last_synced
                .as_deref()
                .map_or_else(String::new, |t| format!(", synced {t}"));
            format!(
                "{} — {} ({}{docs}{synced})\n{}",
                s.id,
                s.name,
                if s.available {
                    "available"
                } else {
                    "unavailable"
                },
                s.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Sources")),
        format!("{} knowledge source(s)", sources.len()),
    )
}

fn format_index_stats(stats: &IndexStats) -> (CliArtifact, String) {
    let body = format!(
        "documents: {}\nchunks: {}\nlast_built: {}\nversion: {}",
        stats.documents,
        stats.chunks,
        stats.last_built.as_deref().unwrap_or("unknown"),
        stats.version.as_deref().unwrap_or("unknown")
    );
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Index Stats")),
        format!("{} documents, {} chunks", stats.documents, stats.chunks),
    )
}

fn to_mcp_error(e: &KnowledgeBankError) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

pub(super) struct SearchKnowledgeHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for SearchKnowledgeHandler<'_> {
    type Input = SearchKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        SEARCH_TOOL
    }

    fn description(&self) -> &'static str {
        "Search the enterprise knowledge bank."
    }

    async fn handle(
        &self,
        input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let filter = SearchFilter {
            source_types: input.source_types.unwrap_or_default(),
            top_k: input.top_k,
        };
        match self.retriever.search(&input.query, &filter).await {
            Ok(hits) => Ok(format_hits(&hits, &input.query)),
            Err(KnowledgeBankError::NotConfigured(reason)) => Ok(not_configured_artifact(&reason)),
            Err(e) => Err(to_mcp_error(&e)),
        }
    }
}

pub(super) struct ListSourcesHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for ListSourcesHandler<'_> {
    type Input = ListKnowledgeSourcesInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        LIST_SOURCES_TOOL
    }

    fn description(&self) -> &'static str {
        "List searchable knowledge sources."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        match self.retriever.list_sources().await {
            Ok(sources) => Ok(format_sources(&sources)),
            Err(KnowledgeBankError::NotConfigured(reason)) => Ok(not_configured_artifact(&reason)),
            Err(e) => Err(to_mcp_error(&e)),
        }
    }
}

pub(super) struct IndexStatsHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for IndexStatsHandler<'_> {
    type Input = IndexStatsInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        INDEX_STATS_TOOL
    }

    fn description(&self) -> &'static str {
        "Report retrieval-index statistics."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        match self.retriever.index_stats().await {
            Ok(stats) => Ok(format_index_stats(&stats)),
            Err(KnowledgeBankError::NotConfigured(reason)) => Ok(not_configured_artifact(&reason)),
            Err(e) => Err(to_mcp_error(&e)),
        }
    }
}

pub(super) async fn authenticate_tool_request(
    db_pool: &DbPool,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
    authz_hook: &SharedAuthzHook,
) -> Result<SysRequestContext, McpError> {
    let server_name = service_id;
    let rbac_result = enforce_rbac_from_registry(ctx, service_id, authz_hook).await;

    match rbac_result {
        Ok(result) => {
            match result.expect_authenticated(
                "BUG: knowledge-bank requires OAuth but auth was not enforced",
            ) {
                Ok(authenticated) => {
                    record_mcp_access(
                        db_pool,
                        authenticated.context.user_id(),
                        server_name,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    Ok(authenticated.context.clone())
                },
                Err(e) => {
                    record_mcp_access_rejected(db_pool, server_name, tool_name, e.message.as_ref())
                        .await;
                    Err(e)
                },
            }
        },
        Err(e) => {
            record_mcp_access_rejected(db_pool, server_name, tool_name, &format!("{e}")).await;
            Err(e)
        },
    }
}

#[doc(hidden)]
#[expect(
    missing_debug_implementations,
    reason = "borrows `dyn KnowledgeRetriever`, which has no Debug bound"
)]
pub struct Dispatch<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
    pub retriever: &'a dyn KnowledgeRetriever,
}

// Why: Exposed (behind `#[doc(hidden)]`) so the external test workspace can
// assert the unknown-tool arm without an rmcp `Peer`, which only exists once
// a transport is serving. Not part of the public API.
#[doc(hidden)]
pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    match tool_name {
        SEARCH_TOOL => {
            let handler = SearchKnowledgeHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        LIST_SOURCES_TOOL => {
            let handler = ListSourcesHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        INDEX_STATS_TOOL => {
            let handler = IndexStatsHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'. Available tools: '{SEARCH_TOOL}', \
                 '{LIST_SOURCES_TOOL}', '{INDEX_STATS_TOOL}'."
            ),
            None,
        )),
    }
}
