//! Per-call logic for the `knowledge-bank` server: authentication, the admin
//! gate on uploads, and the three tool handlers.
//!
//! Read tools (`search_project_context`, `list_documents`) are open to any
//! role the registry grants the server to; `upload_document` additionally
//! requires the admin role on the authenticated user — the same double-gate
//! pattern the astound-admin surface uses, and the contract the real RAG
//! server inherits.

use crate::store::{Document, KnowledgeStore};
use crate::tools::{ListInput, SearchInput, TOOL_LIST, TOOL_SEARCH, TOOL_UPLOAD, UploadInput};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::{RequestContext, RoleServer};
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

const DEFAULT_SEARCH_LIMIT: usize = 5;

/// Render matched documents as the markdown body returned to the model.
///
/// Exposed (behind `#[doc(hidden)]`) so the external test workspace can assert
/// the empty-result sentinel and the per-document heading shape directly; not
/// part of the public API.
#[doc(hidden)]
pub fn document_summary(documents: &[Document]) -> String {
    if documents.is_empty() {
        return "No matching documents in the knowledge bank.".to_owned();
    }
    documents
        .iter()
        .map(|d| {
            format!(
                "## {} ({}, {}, {})\n\n{}",
                d.title, d.doc_type, d.project, d.date, d.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn text_artifact(title: &str, body: &str) -> CliArtifact {
    CliArtifact::text(TextArtifact::new(body).with_title(title))
}

struct SearchHandler {
    store: Arc<KnowledgeStore>,
}

impl McpToolHandler for SearchHandler {
    type Input = SearchInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_SEARCH
    }

    fn description(&self) -> &'static str {
        "Search the project knowledge bank."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let limit = input
            .limit
            .map_or(DEFAULT_SEARCH_LIMIT, |l| l.clamp(1, 20) as usize);
        let results = self
            .store
            .search(&input.query, input.project_id.as_deref(), limit);
        let summary = format!("{} document(s) matched \"{}\"", results.len(), input.query);
        let body = document_summary(&results);
        std::future::ready(Ok((
            text_artifact("Project Context Search", &body),
            summary,
        )))
    }
}

struct ListHandler {
    store: Arc<KnowledgeStore>,
}

impl McpToolHandler for ListHandler {
    type Input = ListInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_LIST
    }

    fn description(&self) -> &'static str {
        "List knowledge bank documents."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let documents = self
            .store
            .list_documents(input.project_id.as_deref(), input.doc_type.as_deref());
        let summary = format!("{} document(s) in the knowledge bank", documents.len());
        let listing = documents
            .iter()
            .map(|d| {
                format!(
                    "- {} — {} ({}, {}, {})",
                    d.id, d.title, d.doc_type, d.project, d.date
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let body = if listing.is_empty() {
            "The knowledge bank holds no documents matching the filter.".to_owned()
        } else {
            listing
        };
        std::future::ready(Ok((
            text_artifact("Knowledge Bank Documents", &body),
            summary,
        )))
    }
}

struct UploadHandler {
    store: Arc<KnowledgeStore>,
}

impl McpToolHandler for UploadHandler {
    type Input = UploadInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_UPLOAD
    }

    fn description(&self) -> &'static str {
        "Upload a document to the knowledge bank (admin only)."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let slug: String = input
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let id = format!("{}-{}", input.doc_type, slug.trim_matches('-'));
        let document = Document {
            id: id.clone(),
            doc_type: input.doc_type,
            project: input.project_id,
            title: input.title,
            date: chrono::Utc::now().date_naive().to_string(),
            content: input.content,
        };
        self.store.insert(document);
        let summary = format!("Document {id} uploaded to the knowledge bank");
        std::future::ready(Ok((text_artifact("Document Uploaded", &summary), summary)))
    }
}

pub(super) async fn authenticate_tool_request(
    db_pool: &DbPool,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
    authz_hook: &SharedAuthzHook,
) -> Result<SysRequestContext, McpError> {
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
                        service_id,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    Ok(authenticated.context.clone())
                },
                Err(e) => {
                    record_mcp_access_rejected(db_pool, service_id, tool_name, e.message.as_ref())
                        .await;
                    Err(e)
                },
            }
        },
        Err(e) => {
            record_mcp_access_rejected(db_pool, service_id, tool_name, &format!("{e}")).await;
            Err(e)
        },
    }
}

/// The admin gate on `upload_document`.
///
/// Exposed (behind `#[doc(hidden)]`) so the external test workspace can assert
/// the gate without a live MCP transport; not part of the public API.
#[doc(hidden)]
pub fn require_admin(request_context: &SysRequestContext) -> Result<(), McpError> {
    let is_admin = request_context
        .user
        .as_ref()
        .is_some_and(AuthenticatedUser::is_admin);
    if is_admin {
        Ok(())
    } else {
        Err(McpError::invalid_request(
            "upload_document requires the admin role; your account can search and list but not \
             upload"
                .to_owned(),
            None,
        ))
    }
}

/// Route one authenticated tool call to its handler.
///
/// Exposed (behind `#[doc(hidden)]`) so the external test workspace can drive
/// every branch — the three handlers and the unknown-tool arm — without an
/// rmcp `Peer`, which only exists once a transport is serving. `call_tool`
/// itself is therefore unreachable from a test process; this is the seam that
/// makes its body testable. Not part of the public API.
#[doc(hidden)]
#[derive(Debug)]
pub struct Dispatch<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
}

impl Dispatch<'_> {
    async fn run<H: McpToolHandler>(&self, handler: &H) -> Result<CallToolResult, McpError> {
        self.executor
            .execute(handler, self.request, self.request_context, self.client)
            .await
    }
}

#[doc(hidden)]
pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    store: &Arc<KnowledgeStore>,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    match tool_name {
        TOOL_SEARCH => {
            let handler = SearchHandler {
                store: Arc::clone(store),
            };
            ctx.run(&handler).await
        },
        TOOL_LIST => {
            let handler = ListHandler {
                store: Arc::clone(store),
            };
            ctx.run(&handler).await
        },
        TOOL_UPLOAD => {
            require_admin(ctx.request_context)?;
            let handler = UploadHandler {
                store: Arc::clone(store),
            };
            ctx.run(&handler).await
        },
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'. Available tools: {TOOL_SEARCH}, {TOOL_LIST}, \
                 {TOOL_UPLOAD}."
            ),
            None,
        )),
    }
}
