use crate::tools::{self, CommentInput, PostInput, ReadInput, SearchInput, VoteInput, SERVER_NAME};
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, Meta, PaginatedRequestParams,
    ProtocolVersion, RawResource, ReadResourceRequestParams, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::env;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::services::ui_renderer::{CspPolicy, UiMetadata, MCP_APP_MIME_TYPE};
use systemprompt::mcp::{
    build_experimental_capabilities, McpArtifactRepository, McpResponseBuilder,
};
use systemprompt::models::artifacts::{ListArtifact, ListItem, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt_moltbook_extension::{
    security, CreateCommentRequest, CreatePostRequest, ListPostsQuery, MoltbookClient,
    PostSearchQuery, VoteDirection,
};
use tokio::sync::RwLock;

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../templates/artifact-viewer.html");

#[derive(Clone)]
pub struct MoltbookServer {
    db_pool: DbPool,
    service_id: McpServerId,
    client: Arc<RwLock<Option<MoltbookClient>>>,
}

impl MoltbookServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Result<Self> {
        Ok(Self {
            db_pool,
            service_id,
            client: Arc::new(RwLock::new(None)),
        })
    }

    async fn get_client(&self, api_key: &str) -> Result<MoltbookClient, McpError> {
        let mut client_guard = self.client.write().await;

        if client_guard.is_none() {
            let client = MoltbookClient::new(api_key.to_string()).map_err(|e| {
                McpError::internal_error(format!("Failed to create Moltbook client: {e}"), None)
            })?;
            *client_guard = Some(client);
        }

        client_guard.clone().ok_or_else(|| {
            McpError::internal_error("Failed to get Moltbook client".to_string(), None)
        })
    }

    fn get_api_key_from_env() -> Result<String, McpError> {
        env::var("moltbook_api_key")
            .or_else(|_| env::var("MOLTBOOK_API_KEY"))
            .map_err(|_| {
                McpError::internal_error("moltbook_api_key not found in secrets".to_string(), None)
            })
    }
}

impl ServerHandler for MoltbookServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_experimental_with(build_experimental_capabilities())
                .build(),
            server_info: Implementation {
                name: format!("Moltbook ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("Moltbook MCP Server".to_string()),
                website_url: Some("https://www.moltbook.com".to_string()),
            },
            instructions: Some(
                "Moltbook tools for AI agent social network interaction.\n\n\
                Available tools:\n\
                - moltbook_post: Create a new post in a submolt\n\
                - moltbook_comment: Reply to a post or comment\n\
                - moltbook_read: Read posts from feed or submolt\n\
                - moltbook_vote: Upvote or downvote content\n\
                - moltbook_search: Search for posts\n\n\
                Rate limits: 100 req/min, 1 post/30 min, 50 comments/hour"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("Moltbook MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tool_list = tools::list_tools();
        Ok(ListToolsResult {
            tools: tool_list,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str())
            .await?
            .expect_authenticated("BUG: moltbook requires OAuth but auth was not enforced")?;

        let request_context = auth_result.context.clone();
        let mcp_execution_id = McpExecutionId::generate();

        let api_key = Self::get_api_key_from_env()?;
        let client = self.get_client(&api_key).await?;

        let arguments = request.arguments.clone().unwrap_or_default();

        match tool_name.as_str() {
            "moltbook_post" => {
                self.handle_post(arguments, &client, &request_context, &mcp_execution_id)
                    .await
            }
            "moltbook_comment" => {
                self.handle_comment(arguments, &client, &request_context, &mcp_execution_id)
                    .await
            }
            "moltbook_read" => {
                self.handle_read(arguments, &client, &request_context, &mcp_execution_id)
                    .await
            }
            "moltbook_vote" => {
                self.handle_vote(arguments, &client, &request_context, &mcp_execution_id)
                    .await
            }
            "moltbook_search" => {
                self.handle_search(arguments, &client, &request_context, &mcp_execution_id)
                    .await
            }
            _ => Ok(McpResponseBuilder::<()>::build_error(format!(
                "Unknown tool: '{tool_name}'\n\n\
                Available tools: moltbook_post, moltbook_comment, moltbook_read, \
                moltbook_vote, moltbook_search"
            ))),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resource = Resource {
            raw: RawResource {
                uri: format!("ui://{SERVER_NAME}/artifact-viewer"),
                name: "Artifact Viewer".to_string(),
                title: Some("Moltbook Feed Viewer".to_string()),
                description: Some(
                    "Interactive UI viewer for Moltbook social network artifacts. Displays posts, \
                     comments, search results, and feed content with rich formatting. Template \
                     receives artifact data dynamically via MCP Apps ui/notifications/tool-result."
                        .to_string(),
                ),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                size: Some(ARTIFACT_VIEWER_TEMPLATE.len() as u32),
                icons: None,
                meta: None,
            },
            annotations: None,
        };

        Ok(ListResourcesResult {
            resources: vec![resource],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;
        let expected_uri = format!("ui://{SERVER_NAME}/artifact-viewer");

        if uri != &expected_uri {
            return Err(McpError::invalid_params(
                format!("Unknown resource URI: {uri}. Expected: {expected_uri}"),
                None,
            ));
        }

        let ui_meta = UiMetadata::for_static_template(SERVER_NAME)
            .with_csp(CspPolicy::strict())
            .with_prefers_border(true);

        let resource_meta = ui_meta.to_resource_meta();
        let meta = Meta(resource_meta.to_meta_map());

        let contents = ResourceContents::TextResourceContents {
            uri: uri.clone(),
            mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
            text: ARTIFACT_VIEWER_TEMPLATE.to_string(),
            meta: Some(meta),
        };

        Ok(ReadResourceResult {
            contents: vec![contents],
        })
    }
}

impl MoltbookServer {
    async fn handle_post(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        client: &MoltbookClient,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: PostInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let sanitized_title = security::validate_and_sanitize(&input.title).map_err(|e| {
            McpError::invalid_params(format!("Content validation failed: {e}"), None)
        })?;

        let sanitized_content = security::validate_and_sanitize(&input.content).map_err(|e| {
            McpError::invalid_params(format!("Content validation failed: {e}"), None)
        })?;

        let request = CreatePostRequest {
            submolt: input.submolt.clone(),
            title: sanitized_title,
            content: sanitized_content,
            url: None,
        };

        let response = client.create_post(request).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to create Moltbook post");
            McpError::internal_error(format!("Failed to create post: {e}"), None)
        })?;

        let summary = format!(
            "Post created successfully!\nPost ID: {}\nURL: https://www.moltbook.com/posts/{}\nSubmolt: {}",
            response.id, response.id, input.submolt
        );

        let artifact = TextArtifact::new(&summary, ctx).with_title("Moltbook Post Created");

        McpResponseBuilder::new(artifact, "moltbook_post", ctx, execution_id)
            .build(&summary)
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }

    async fn handle_comment(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        client: &MoltbookClient,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: CommentInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let sanitized_content = security::validate_and_sanitize(&input.content).map_err(|e| {
            McpError::invalid_params(format!("Content validation failed: {e}"), None)
        })?;

        let request = CreateCommentRequest {
            post_id: input.post_id.clone(),
            content: sanitized_content,
            parent_id: input.parent_id.clone(),
        };

        let response = client.create_comment(request).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to create Moltbook comment");
            McpError::internal_error(format!("Failed to create comment: {e}"), None)
        })?;

        let summary = format!(
            "Comment created successfully!\nComment ID: {}\nPost ID: {}",
            response.id, input.post_id
        );

        let artifact = TextArtifact::new(&summary, ctx).with_title("Moltbook Comment Created");

        McpResponseBuilder::new(artifact, "moltbook_comment", ctx, execution_id)
            .build(&summary)
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }

    async fn handle_read(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        client: &MoltbookClient,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: ReadInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let limit = input.limit.unwrap_or(25).min(100);

        let posts = if let Some(submolt) = &input.submolt {
            let query = ListPostsQuery {
                submolt: Some(submolt.clone()),
                limit: Some(limit),
                ..Default::default()
            };
            client.list_posts(query).await
        } else {
            client.get_feed(Some(limit)).await
        }
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to read Moltbook posts");
            McpError::internal_error(format!("Failed to read posts: {e}"), None)
        })?;

        let items: Vec<ListItem> = posts
            .into_iter()
            .map(|post| {
                let safe_content = security::sanitize_content(&post.content);
                let safe_title = security::sanitize_content(&post.title);
                ListItem::new(
                    safe_title,
                    safe_content,
                    format!("https://www.moltbook.com/posts/{}", post.id),
                )
                .with_id(&post.id)
                .with_category(&post.submolt)
            })
            .collect();

        let count = items.len();
        let artifact = ListArtifact::new(ctx).with_items(items);

        let summary = format!("Found {} posts in feed", count);

        let artifact_repo = McpArtifactRepository::new(&self.db_pool).map_err(|e| {
            McpError::internal_error(format!("Failed to create artifact repository: {e}"), None)
        })?;

        McpResponseBuilder::new(artifact, "moltbook_read", ctx, execution_id)
            .build_and_persist(
                summary.clone(),
                &artifact_repo,
                "list",
                Some("Moltbook Feed".to_string()),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }

    async fn handle_vote(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        client: &MoltbookClient,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: VoteInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let direction = match input.direction.to_lowercase().as_str() {
            "up" => VoteDirection::Up,
            "down" => VoteDirection::Down,
            _ => {
                return Err(McpError::invalid_params(
                    "Invalid vote direction. Use 'up' or 'down'.".to_string(),
                    None,
                ))
            }
        };

        client
            .vote_post(&input.post_id, direction)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to vote on Moltbook post");
                McpError::internal_error(format!("Failed to vote: {e}"), None)
            })?;

        let summary = format!(
            "Vote recorded successfully!\nPost ID: {}\nDirection: {}",
            input.post_id, input.direction
        );

        let artifact = TextArtifact::new(&summary, ctx).with_title("Moltbook Vote");

        McpResponseBuilder::new(artifact, "moltbook_vote", ctx, execution_id)
            .build(&summary)
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }

    async fn handle_search(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        client: &MoltbookClient,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: SearchInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let limit = input.limit.unwrap_or(25).min(100);

        let query = PostSearchQuery {
            query: input.query.clone(),
            submolt: input.submolt.clone(),
            limit: Some(limit),
        };

        let posts = client.search_posts(query).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to search Moltbook posts");
            McpError::internal_error(format!("Failed to search: {e}"), None)
        })?;

        let items: Vec<ListItem> = posts
            .into_iter()
            .map(|post| {
                let safe_content = security::sanitize_content(&post.content);
                let safe_title = security::sanitize_content(&post.title);
                ListItem::new(
                    safe_title,
                    safe_content,
                    format!("https://www.moltbook.com/posts/{}", post.id),
                )
                .with_id(&post.id)
                .with_category(&post.submolt)
            })
            .collect();

        let count = items.len();
        let artifact = ListArtifact::new(ctx).with_items(items);

        let summary = format!("Found {} posts matching '{}'", count, input.query);

        let artifact_repo = McpArtifactRepository::new(&self.db_pool).map_err(|e| {
            McpError::internal_error(format!("Failed to create artifact repository: {e}"), None)
        })?;

        McpResponseBuilder::new(artifact, "moltbook_search", ctx, execution_id)
            .build_and_persist(
                summary.clone(),
                &artifact_repo,
                "list",
                Some("Moltbook Search".to_string()),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }
}
