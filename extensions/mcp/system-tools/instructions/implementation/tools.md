# Tool Implementation

Patterns for implementing MCP tools with type-safe dispatch.

---

## Tool Execution Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Request Received                                         │
│    └─ call_tool(CallToolRequestParam, ctx)                  │
├─────────────────────────────────────────────────────────────┤
│ 2. Extract Progress Token                                   │
│    └─ ctx.meta.get_progress_token() → ProgressCallback      │
├─────────────────────────────────────────────────────────────┤
│ 3. Input Validation                                         │
│    └─ parse_tool_input<T>(request) → Result<T, McpError>    │
├─────────────────────────────────────────────────────────────┤
│ 4. Create Execution Record                                  │
│    └─ tool_repo.start_execution(request) → McpExecutionId   │
├─────────────────────────────────────────────────────────────┤
│ 5. Execute Handler (with progress steps)                    │
│    ├─ report_progress(0.0, "Starting...")                   │
│    ├─ load skills → inject into prompt                      │
│    ├─ call AI service (if needed)                           │
│    ├─ report_progress(50.0, "Processing...")                │
│    └─ report_progress(100.0, "Complete")                    │
├─────────────────────────────────────────────────────────────┤
│ 6. Build Response                                           │
│    └─ build_tool_response(result, metadata)                 │
├─────────────────────────────────────────────────────────────┤
│ 7. Complete Execution Record                                │
│    └─ tool_repo.complete_execution(execution_id, result)    │
└─────────────────────────────────────────────────────────────┘
```

---

## ToolHandler Trait

```rust
use async_trait::async_trait;
use rmcp::model::{CallToolResult, Tool};
use serde::de::DeserializeOwned;

pub struct ToolContext<'a> {
    pub execution_id: &'a McpExecutionId,
    pub db_pool: &'a DbPool,
    pub ai_service: &'a AiService,
    pub skill_loader: &'a SkillService,
    pub request: RequestContext,
    pub progress: Option<ProgressCallback>,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    const NAME: &'static str;
    type Input: DeserializeOwned + Send;

    fn tool() -> Tool;

    async fn execute(
        &self,
        input: Self::Input,
        ctx: ToolContext<'_>,
    ) -> Result<CallToolResult, McpError>;
}
```

---

## Input Parsing

```rust
use serde::de::DeserializeOwned;

#[must_use]
fn parse_tool_input<T: DeserializeOwned>(
    request: &CallToolRequestParam,
) -> Result<T, McpError> {
    let args = request.arguments.clone().unwrap_or_default();
    serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))
}
```

---

## Response Building

```rust
use systemprompt::models::artifacts::{ExecutionMetadata, ToolResponse};
use rmcp::model::{CallToolResult, Content};

#[must_use]
fn build_tool_response<T: Serialize>(
    tool_name: &str,
    result: T,
    summary: impl Into<String>,
    execution_id: &McpExecutionId,
) -> CallToolResult {
    let metadata = ExecutionMetadata::new().tool(tool_name);
    let artifact_id = Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        execution_id.clone(),
        result,
        metadata.clone(),
    );

    CallToolResult {
        content: vec![Content::text(summary.into())],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    }
}
```

---

## Server Handler

```rust
impl ServerHandler for ContentServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.as_ref();

        let progress = ctx.meta.get_progress_token()
            .map(|token| create_progress_callback(token.clone(), ctx.peer.clone()));

        let execution_id = self.tool_repo
            .start_execution(&ToolExecutionRequest {
                tool_name: tool_name.to_string(),
                mcp_server_name: self.service_id.to_string(),
                started_at: Utc::now(),
            })
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let tool_ctx = ToolContext {
            execution_id: &execution_id,
            db_pool: &self.db_pool,
            ai_service: &self.ai_service,
            skill_loader: &self.skill_loader,
            request: request_context,
            progress,
        };

        let result = self.registry.dispatch(tool_name, request, tool_ctx).await;

        tokio::spawn({
            let tool_repo = self.tool_repo.clone();
            let execution_id = execution_id.clone();
            let status = if result.is_ok() { "success" } else { "failed" };
            async move {
                tool_repo
                    .complete_execution(&execution_id, &ToolExecutionResult {
                        status: status.to_string(),
                        completed_at: Utc::now(),
                        ..Default::default()
                    })
                    .await
                    .inspect_err(|e| tracing::error!(error = %e, "Failed to complete execution"))
                    .ok();
            }
        });

        result
    }
}
```

---

## See Also

- [progress.md](./progress.md) - Progress reporting
- [skills.md](./skills.md) - Skills integration
- [errors.md](./errors.md) - Error handling
