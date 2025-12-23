# Error Handling

Patterns for domain errors, MCP error conversion, and error propagation.

---

## Domain Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}
```

---

## MCP Error Conversion

```rust
impl From<ToolError> for McpError {
    fn from(err: ToolError) -> Self {
        match err {
            ToolError::InvalidInput(msg) | ToolError::NotFound(msg) => {
                McpError::invalid_params(msg, None)
            }
            _ => McpError::internal_error(err.to_string(), None),
        }
    }
}
```

---

## Error Context Chain

```rust
use anyhow::Context;

let skill = skill_loader
    .load_skill(skill_id, &ctx)
    .await
    .context("Failed to load skill")?;

let result = ai_service
    .generate(&request)
    .await
    .context("AI generation failed")?;
```

---

## Timeout Wrapper

```rust
use tokio::time::{timeout, Duration};

const TOOL_TIMEOUT: Duration = Duration::from_secs(30);

#[must_use]
async fn execute_with_timeout<F, T, E>(operation: F) -> Result<T, McpError>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    timeout(TOOL_TIMEOUT, operation)
        .await
        .map_err(|_| McpError::internal_error("Operation timed out", None))?
        .map_err(|e| McpError::internal_error(format!("{e}"), None))
}
```

---

## Background Task Error Handling

```rust
tokio::spawn({
    let db_pool = db_pool.clone();
    let execution_id = execution_id.clone();
    async move {
        if let Err(e) = complete_execution(&db_pool, &execution_id).await {
            tracing::error!(error = %e, "Failed to complete execution record");
        }
    }
});
```

---

## Concurrent Operations

```rust
let (files_result, db_result) = tokio::join!(
    self.sync_files(direction, dry_run),
    self.sync_database(direction, dry_run, None),
);

let files = files_result.map_err(|e| {
    tracing::error!(error = %e, "File sync failed");
    e
})?;

let db = db_result.map_err(|e| {
    tracing::error!(error = %e, "Database sync failed");
    e
})?;
```

---

## Error Rules

| Rule | Description |
|------|-------------|
| Never swallow | Always log before `.ok()` |
| Log once | Log at handling boundary, not every propagation |
| No stack traces | Never expose internal traces to clients |
| Descriptive messages | Provide actionable error messages |

### Never Swallow Errors

```rust
operation()
    .await
    .inspect_err(|e| tracing::error!(error = %e, "Operation failed"))
    .ok();
```

---

## See Also

- [tools.md](./tools.md) - Tool implementation
- [progress.md](./progress.md) - Progress reporting
- [skills.md](./skills.md) - Skills integration
