# MCP Extension Boundaries

This document defines module boundaries, acceptable integration patterns, and communication rules for MCP extensions.

---

## Extension Isolation

Each MCP server extension operates as an **isolated, standalone crate**:

| Rule | Description |
|------|-------------|
| Standalone Crate | Each extension has its own `Cargo.toml` |
| Workspace Exclusion | Extensions are excluded from the main workspace |
| Independent Lock | Each extension maintains its own `Cargo.lock` |
| No Cross-Import | Extensions NEVER import from other extensions |

### Workspace Configuration

Extensions are excluded from the root workspace:

```toml
[workspace]
members = [
    # workspace members...
]
exclude = [
    "extensions/mcp/admin",
    "extensions/mcp/infrastructure",
    "extensions/mcp/system-tools",
]
```

---

## Dependency Direction

Dependencies flow in one direction only:

```
┌─────────────────────────────────────────────────────────┐
│                    MCP Extension                        │
│  (systemprompt-infrastructure, systemprompt-admin, etc) │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  systemprompt facade                    │
│              (unified public API)                       │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              systemprompt-core modules                  │
│  (database, logging, mcp, system, identifiers, etc)     │
└─────────────────────────────────────────────────────────┘
```

### Strict Rules

| Direction | Status |
|-----------|--------|
| Extension → Facade | **YES (required)** |
| Extension → rmcp | **YES (required)** |
| Extension → Extension | **NEVER** |
| Facade → Extension | **NEVER** |
| Core → Extension | **NEVER** |

---

## Acceptable Import Patterns

### Preferred: Use the Facade

```rust
use systemprompt::system::AppContext;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::models::Config;
```

### Acceptable: Direct Core Module Imports

```rust
use systemprompt_core_mcp::create_router;
use systemprompt_core_logging::init_logging;
use systemprompt_core_database::DbPool;
```

### Required: rmcp Protocol Types

```rust
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content,
        ListToolsResult, Tool, ServerInfo,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer, ServerHandler,
};
```

### Import Pattern Summary

| Pattern | Status | Use Case |
|---------|--------|----------|
| `use systemprompt::*` | Preferred | Most imports |
| `use systemprompt_core_*::*` | Acceptable | Specific core modules |
| `use rmcp::*` | Required | MCP protocol types |
| `use other_extension::*` | **NEVER** | Cross-extension import |

---

## Communication Boundaries

### Cross-Extension Communication

Extensions MUST NOT communicate directly. Use these patterns instead:

| Method | Description | Use Case |
|--------|-------------|----------|
| Database | Shared PostgreSQL tables | Persistent data exchange |
| HTTP API | RESTful endpoints | Real-time requests |
| Events | Database-backed events | Async notifications |

**NEVER**:
- Import types from another extension
- Call functions from another extension
- Share memory between extensions

### RBAC Boundary

Authorization MUST be enforced at the tool call entry point:

RBAC check MUST be the first operation in `call_tool`:

```rust
async fn call_tool(
    &self,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let request_context = enforce_rbac_from_registry(
        &ctx,
        self.service_id.as_ref(),
        self.app_context.clone(),
    )
    .await?;

    handle_tool_call(request, request_context).await
}
```

### Execution Tracking Boundary

All tool executions MUST be recorded via `ToolUsageRepository`.

**Start execution:**

```rust
let exec_request = ToolExecutionRequest {
    tool_name: tool_name.clone(),
    mcp_server_name: self.service_id.to_string(),
    input: serde_json::Value::Object(arguments),
    started_at: Utc::now(),
};
let execution_id = tool_repo.start_execution(&exec_request).await?;
```

**Execute tool:**

```rust
let result = handle_tool_call(request, request_context).await;
```

**Complete execution:**

```rust
let exec_result = ToolExecutionResult {
    output: result.structured_content,
    status: if result.is_ok() { "success" } else { "failed" },
    completed_at: Utc::now(),
};
tool_repo.finish_execution(&execution_id, &exec_result).await?;
```

---

## Error Handling Boundary

### Internal vs Protocol Errors

| Layer | Error Type | Handling |
|-------|------------|----------|
| Internal | `anyhow::Result<T>` | Propagate within extension |
| Protocol | `rmcp::ErrorData` (McpError) | Return to MCP client |

### Error Conversion

Convert internal errors to protocol errors at the handler boundary:

```rust
pub async fn handle_my_tool(
    service: &MyService,
    request: CallToolRequestParam,
) -> Result<CallToolResult, McpError> {
    let result = service.do_work().await
        .map_err(|e| McpError::internal_error(
            format!("Operation failed: {e}"),
            None
        ))?;

    Ok(build_tool_response("my_tool", result, "Operation complete", &execution_id))
}
```

### McpError Types

| Error Type | Use Case |
|------------|----------|
| `McpError::invalid_params()` | Invalid input parameters |
| `McpError::method_not_found()` | Unknown tool/prompt name |
| `McpError::internal_error()` | Service/database failures |

### Error Propagation Rules

| Rule | Description |
|------|-------------|
| Convert at boundary | Map internal errors to McpError at handler level |
| Log once | Log errors at the handling boundary, not at every propagation |
| No stack traces | NEVER include internal stack traces in client responses |
| Descriptive messages | Provide actionable error messages for clients |

---

## Resource Boundary

### Resources vs Tools

| Capability | Purpose | Access Pattern |
|------------|---------|----------------|
| Resources | Read-only data exposure | Direct read via `read_resource()` |
| Tools | Operations with side effects | Execute via `call_tool()` |

### Resource Management Pattern

Resources that require complex logic or side effects SHOULD be exposed through tools:

Resources managed via tools return empty lists:

```rust
pub async fn list_resources(
    &self,
    _request: Option<PaginatedRequestParam>,
    _ctx: RequestContext<RoleServer>,
) -> Result<ListResourcesResult, McpError> {
    Ok(ListResourcesResult {
        resources: Vec::new(),
        next_cursor: None,
    })
}

pub async fn read_resource(
    &self,
    _request: ReadResourceRequestParam,
    _ctx: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    Err(McpError::invalid_params(
        "Resources are managed through sync tools. Use sync_files or sync_database.",
        None
    ))
}
```

---

## Testing Boundaries

### Test Location

| Test Type | Location | Scope |
|-----------|----------|-------|
| Unit Tests | `tests/` directory | Single function/module |
| Integration Tests | `tests/` directory | Cross-module |

**NEVER** use `#[cfg(test)]` inline modules in source files.

### Test Dependencies

Tests MAY import from the extension crate:

**tests/tool_tests.rs:**

```rust
use systemprompt_infrastructure::{InfrastructureServer, create_database_connection};
```

---

## See Also

- [overview.md](./overview.md) - Extension architecture overview
- [../implementation/prompts.md](../implementation/prompts.md) - Prompt implementation
- [../implementation/tools.md](../implementation/tools.md) - Tool implementation
