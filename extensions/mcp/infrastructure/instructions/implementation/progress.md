# Progress Reporting

Long-running tools must report progress via MCP notifications.

---

## ProgressCallback Type

```rust
use rmcp::model::{ProgressNotificationParam, ProgressToken};
use rmcp::service::Peer;
use std::future::Future;
use std::pin::Pin;

pub type ProgressCallback = Box<
    dyn Fn(f64, Option<f64>, Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send + Sync,
>;
```

---

## Create Progress Callback

```rust
#[must_use]
fn create_progress_callback(token: ProgressToken, peer: Peer<RoleServer>) -> ProgressCallback {
    Box::new(move |progress, total, message| {
        let token = token.clone();
        let peer = peer.clone();
        Box::pin(async move {
            let _ = peer.notify_progress(ProgressNotificationParam {
                progress_token: token,
                progress,
                total,
                message,
            }).await;
        })
    })
}
```

---

## Report Progress Helper

```rust
async fn report_progress(progress: &Option<ProgressCallback>, value: f64, message: &str) {
    if let Some(ref notify) = progress {
        notify(value, Some(100.0), Some(message.to_string())).await;
    }
}
```

---

## Usage Pattern

```rust
pub async fn handle_research(
    ctx: ToolContext<'_>,
) -> Result<CallToolResult, McpError> {
    report_progress(&ctx.progress, 0.0, "Starting research...").await;

    let skill_content = ctx.skill_loader.load_skill("research", &ctx.request).await?;
    report_progress(&ctx.progress, 10.0, "Loaded skill, querying AI...").await;

    let result = ctx.ai_service.generate(&request).await?;
    report_progress(&ctx.progress, 70.0, "Processing results...").await;

    let formatted = process_results(result)?;
    report_progress(&ctx.progress, 100.0, "Research complete").await;

    Ok(build_tool_response("research", formatted, "Research complete", ctx.execution_id))
}
```

---

## Progress Step Guidelines

| Stage | Progress | Message |
|-------|----------|---------|
| Start | 0.0 | "Starting {operation}..." |
| Skill loaded | 10.0 | "Loaded skill, querying AI..." |
| AI response | 50-70.0 | "Processing results..." |
| Formatting | 80-90.0 | "Formatting output..." |
| Complete | 100.0 | "{Operation} complete" |

---

## See Also

- [tools.md](./tools.md) - Tool implementation
- [skills.md](./skills.md) - Skills integration
