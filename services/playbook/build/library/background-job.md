---
title: "Add Background Job"
description: "Add scheduled background tasks to your extension."
author: "SystemPrompt"
slug: "build-02-library-extensions-add-background-job"
keywords: "jobs, background, scheduler, cron"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Add Background Job

Add scheduled tasks to your extension. Reference: `extensions/web/src/jobs/` for examples.

> **Help**: `{ "command": "core playbooks show build_add-background-job" }`

---

## Structure

```
extensions/my-extension/src/
├── jobs/
│   ├── mod.rs
│   └── cleanup.rs
└── extension.rs
```

---

## Create Job

File: `src/jobs/cleanup.rs`. See `extensions/web/src/jobs/` for reference.

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_provider_contracts::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct CleanupJob;

#[async_trait::async_trait]
impl Job for CleanupJob {
    fn name(&self) -> &'static str {
        "my-extension-cleanup"
    }

    fn description(&self) -> &'static str {
        "Clean up expired items"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<Arc<PgPool>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let deleted = sqlx::query!(
            "DELETE FROM my_items WHERE created_at < NOW() - INTERVAL '30 days'"
        )
        .execute(&*pool)
        .await?
        .rows_affected();

        tracing::info!(deleted = deleted, "Cleanup completed");

        Ok(JobResult::success().with_message(format!("Deleted {} items", deleted)))
    }
}
```

---

## Register Job

In `src/extension.rs`. See `extensions/web/src/extension.rs:75-80` for reference.

```rust
fn jobs(&self) -> Vec<Arc<dyn Job>> {
    vec![
        Arc::new(CleanupJob),
    ]
}
```

---

## Cron Schedule

6-field format: `second minute hour day month weekday`

| Schedule | Meaning |
|----------|---------|
| `0 0 * * * *` | Every hour |
| `0 */15 * * * *` | Every 15 minutes |
| `0 0 0 * * *` | Daily at midnight |
| `0 30 2 * * *` | Daily at 2:30 AM |
| `0 0 0 * * 1` | Every Monday |

---

## Override Schedule

In `profile.yaml`:

```yaml
scheduler:
  jobs:
    - extension: my-extension
      job: my-extension-cleanup
      schedule: "0 */30 * * * *"
      enabled: true
```

---

## Run Manually

```bash
systemprompt infra jobs run my-extension-cleanup
```

---

## Checklist

- [ ] `src/jobs/` directory with job files
- [ ] Implements `Job` trait
- [ ] Unique `name()` with extension prefix
- [ ] Valid 6-field cron `schedule()`
- [ ] Gets pool from `ctx.db_pool()`
- [ ] Returns `JobResult::success()` or error
- [ ] Registered in `jobs()` method
- [ ] Uses structured logging with `tracing`

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Job not running | Check schedule syntax |
| Job disabled | Set `enabled: true` in profile |
| Database error | Verify pool access pattern |

---

## Quick Reference

| Task | Command |
|------|---------|
| List jobs | `systemprompt infra jobs list` |
| Run job | `systemprompt infra jobs run <name>` |
| Check status | `systemprompt infra jobs status` |

---

## Related

-> See [Create Library Extension](create-extension.md) for full extension setup
-> See [Job Extension](../../documentation/extensions/traits/job-extension.md) for trait reference
-> See [Rust Standards](../06-standards/rust-standards.md) for code style