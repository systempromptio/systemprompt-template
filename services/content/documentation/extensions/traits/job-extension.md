---
title: "Job Extension"
description: "Add background jobs and scheduled tasks to your extension."
author: "SystemPrompt Team"
slug: "extensions/traits/job-extension"
keywords: "jobs, background, scheduler, cron, tasks"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Job Extension

Extensions add background tasks via the `jobs()` method.

## Jobs Method

```rust
fn jobs(&self) -> Vec<Arc<dyn Job>> {
    vec![
        Arc::new(CleanupJob),
        Arc::new(SyncJob),
    ]
}
```

## Job Trait

```rust
use systemprompt_provider_contracts::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct CleanupJob;

#[async_trait]
impl Job for CleanupJob {
    fn name(&self) -> &'static str {
        "cleanup"
    }

    fn description(&self) -> &'static str {
        "Clean up expired records"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"  // Every hour
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<Arc<PgPool>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let deleted = sqlx::query!(
            "DELETE FROM temp_records WHERE expires_at < NOW()"
        )
        .execute(&*pool)
        .await?
        .rows_affected();

        Ok(JobResult::success().with_message(format!("Deleted {} records", deleted)))
    }
}
```

## Cron Schedule Format

6-field cron expression: `second minute hour day-of-month month day-of-week`

```
┌───────────── second (0-59)
│ ┌───────────── minute (0-59)
│ │ ┌───────────── hour (0-23)
│ │ │ ┌───────────── day of month (1-31)
│ │ │ │ ┌───────────── month (1-12)
│ │ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │ │
* * * * * *
```

Examples:
- `0 0 * * * *` - Every hour at minute 0
- `0 */15 * * * *` - Every 15 minutes
- `0 0 0 * * *` - Daily at midnight
- `0 30 2 * * *` - Daily at 2:30 AM
- `0 0 0 * * 1` - Every Monday at midnight

## JobContext

```rust
async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
    // Get database pool
    let pool = ctx.db_pool::<Arc<PgPool>>()?;

    // Get configuration
    let config = ctx.config();

    // Access other services
    // ...
}
```

## JobResult

```rust
// Success
Ok(JobResult::success())
Ok(JobResult::success().with_message("Processed 100 items"))

// Failure (will be retried based on configuration)
Err(anyhow::anyhow!("Database connection failed"))
```

## Configuration Override

Override job schedules in `profile.yaml`:

```yaml
scheduler:
  jobs:
    - extension: my-extension
      job: cleanup
      schedule: "0 */30 * * * *"  # Override to every 30 minutes
      enabled: true
```

## CLI Commands

```bash
# Run job manually
systemprompt infra jobs run cleanup

# List all jobs
systemprompt infra jobs list

# Show job status
systemprompt infra jobs status cleanup
```

## Typed Extension

```rust
use systemprompt::extension::prelude::JobExtensionTyped;

impl JobExtensionTyped for MyExtension {
    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(CleanupJob)]
    }
}
```