---
title: "Creating Background Jobs"
description: "Step-by-step guide to creating Job implementations for scheduled tasks and async processing."
author: "SystemPrompt Team"
slug: "build-job"
keywords: "job, scheduler, background tasks, cron, async"
image: "/files/images/playbooks/build-job.svg"
kind: "playbook"
public: true
tags: ["build", "jobs", "scheduler"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Create a Job struct and implement the Job trait"
  - "Configure cron schedules for automated execution"
  - "Access database and services from JobContext"
  - "Return proper JobResult with statistics"
related_code:
  - title: "Job Trait Definition"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/traits/src/job.rs#L1-L80"
  - title: "Content Ingestion Job Example"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/jobs/ingestion.rs"
related_docs:
  - title: "Background Jobs Reference"
    url: "/documentation/extensions/jobs"
  - title: "Extension Lifecycle Hooks"
    url: "/documentation/extensions/hooks"
---

# Creating Background Jobs

This playbook walks you through creating a background Job that runs on a schedule or on-demand via CLI.

## Prerequisites

- Existing extension crate in `extensions/`
- Understanding of what task needs to run in the background
- Cron schedule pattern (or empty for manual-only)

## Step 1: Create Your Job Struct

Create a new file in your extension's `jobs/` directory:

```rust
// extensions/web/src/jobs/cleanup.rs
use systemprompt::traits::{Job, JobContext, JobResult};
use anyhow::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct CleanupJob;
```

## Step 2: Implement the Job Trait

Implement the required trait methods:

```rust
#[async_trait::async_trait]
impl Job for CleanupJob {
    fn name(&self) -> &'static str {
        "cleanup_old_records"
    }

    fn description(&self) -> &'static str {
        "Deletes records older than 30 days"
    }

    fn schedule(&self) -> &'static str {
        "0 0 3 * * *"  // 3am daily
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        // Job logic here
        Ok(JobResult::success())
    }
}
```

### Required Methods

| Method | Purpose |
|--------|---------|
| `name()` | Unique identifier used in CLI and logs |
| `description()` | Human-readable description |
| `schedule()` | Cron expression (6-field, seconds included) |
| `execute()` | Async function that performs the work |
| `run_on_startup()` | Whether to execute when scheduler starts (default: false) |

## Step 3: Configure the Cron Schedule

Jobs use 6-field cron expressions (seconds, minutes, hours, day, month, weekday):

| Pattern | Description |
|---------|-------------|
| `0 0 * * * *` | Every hour |
| `0 */15 * * * *` | Every 15 minutes |
| `0 0 0 * * *` | Daily at midnight |
| `0 0 3 * * *` | Daily at 3am |
| `0 0 */2 * * *` | Every 2 hours |
| `0 0 0 * * 0` | Weekly on Sunday |
| `""` | No schedule (manual only) |

```rust
fn schedule(&self) -> &'static str {
    "0 0 3 * * *"  // Runs at 3:00:00 AM every day
}
```

## Step 3b: Configure Startup Behavior

Jobs can run immediately when the scheduler starts using `run_on_startup()`:

```rust
fn run_on_startup(&self) -> bool {
    true  // Runs once at startup, then follows schedule
}
```

**Important**: Jobs only run on startup if BOTH conditions are met:
1. `run_on_startup()` returns `true` in code
2. Job is listed in `services/scheduler/config.yaml` with `enabled: true`

This two-layer design allows:
- Developers to set sensible defaults in code
- Ops teams to enable/disable jobs per environment without code changes

-> See [Scheduler Jobs](../../domain/scheduler/jobs.md) for config format.

## Step 4: Access Database from JobContext

Use `JobContext` to access the database pool:

```rust
async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
    let db = ctx
        .db_pool::<DbPool>()
        .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

    let pool = db
        .pool()
        .ok_or_else(|| anyhow::anyhow!("PgPool not available"))?;

    // Use the pool for queries
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM my_table")
        .fetch_one(pool.as_ref())
        .await?;

    Ok(JobResult::success())
}
```

## Step 5: Return JobResult with Statistics

Report execution results:

```rust
async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
    let start = std::time::Instant::now();

    // ... job logic ...
    let processed = 42u64;
    let errors = 0u64;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(JobResult::success()
        .with_stats(processed, errors)
        .with_duration(duration_ms)
        .with_message(format!("Processed {} items", processed)))
}
```

### JobResult Methods

| Method | Purpose |
|--------|---------|
| `JobResult::success()` | Success with no details |
| `.with_stats(processed, errors)` | Add item counts |
| `.with_duration(ms)` | Add execution time |
| `.with_message(msg)` | Add status message |
| `JobResult::failure(msg)` | Report failure |

## Step 6: Register the Job

Use the `submit_job!` macro at module level:

```rust
// At the end of your job file
systemprompt::traits::submit_job!(&CleanupJob);
```

The job is automatically discovered at startup.

## Step 7: Export from Module

Update `jobs/mod.rs`:

```rust
mod cleanup;

pub use cleanup::CleanupJob;
```

## Step 8: Test via CLI

List jobs to verify registration:

```bash
systemprompt infra jobs list
```

Run manually:

```bash
systemprompt infra jobs run cleanup_old_records
```

View execution history:

```bash
systemprompt infra jobs history --job cleanup_old_records --limit 10
```

## Complete Example

```rust
use std::sync::Arc;
use anyhow::Result;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct CleanupJob;

#[async_trait::async_trait]
impl Job for CleanupJob {
    fn name(&self) -> &'static str {
        "cleanup_old_records"
    }

    fn description(&self) -> &'static str {
        "Deletes records older than 30 days"
    }

    fn schedule(&self) -> &'static str {
        "0 0 3 * * *"  // 3am daily
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let pool = db
            .pool()
            .ok_or_else(|| anyhow::anyhow!("PgPool not available"))?;

        let deleted = sqlx::query!(
            "DELETE FROM logs WHERE created_at < NOW() - INTERVAL '30 days'"
        )
        .execute(pool.as_ref())
        .await?
        .rows_affected();

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(JobResult::success()
            .with_stats(deleted, 0)
            .with_duration(duration_ms)
            .with_message(format!("Deleted {} old records", deleted)))
    }
}

systemprompt::traits::submit_job!(&CleanupJob);
```

## Common Patterns

### On-Demand Only Jobs

Set empty schedule for manual-only execution:

```rust
fn schedule(&self) -> &'static str {
    ""  // No automatic schedule
}
```

### Pipeline Jobs

Jobs that orchestrate other jobs:

```rust
async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
    // Run sub-jobs in sequence
    ContentIngestionJob.execute(ctx).await?;
    CopyExtensionAssetsJob::execute_copy().await?;
    ContentPrerenderJob.execute(ctx).await?;

    Ok(JobResult::success())
}
```

### Error Handling

Return `JobResult::failure()` for recoverable errors:

```rust
async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
    let db = match ctx.db_pool::<DbPool>() {
        Some(db) => db,
        None => return Ok(JobResult::failure("Database not available")),
    };

    // ... rest of logic
}
```

Return `Err()` for unrecoverable errors that should be logged:

```rust
async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
    let critical_data = fetch_required_data()
        .await
        .context("Failed to fetch required data")?;

    // ...
}
```

## Project Structure

```
extensions/web/src/
├── jobs/
│   ├── mod.rs
│   ├── ingestion.rs
│   ├── cleanup.rs
│   └── publish.rs
└── extension.rs
```

## Checklist

- [ ] Created job struct with `#[derive(Debug, Clone, Copy, Default)]`
- [ ] Implemented Job trait with all required methods
- [ ] Set unique `name()` identifier
- [ ] Configured appropriate `schedule()` cron pattern
- [ ] Accessed database via `ctx.db_pool::<DbPool>()`
- [ ] Returned `JobResult` with stats and duration
- [ ] Registered with `submit_job!` macro
- [ ] Exported from `jobs/mod.rs`
- [ ] Verified with `systemprompt infra jobs list`
- [ ] Tested with `systemprompt infra jobs run <name>`
