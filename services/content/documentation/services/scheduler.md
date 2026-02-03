---
title: "Scheduler Services"
description: "Configure background jobs with cron scheduling for automated tasks like content publishing, session cleanup, and database maintenance."
author: "SystemPrompt Team"
slug: "services/scheduler"
keywords: "scheduler, jobs, cron, background, tasks, automation"
image: "/files/images/docs/services-scheduler.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Scheduler Services

**TL;DR:** The scheduler runs background jobs on cron schedules. It handles tasks like publishing content, cleaning up sessions, and maintaining the database. Jobs are defined in YAML and can come from the core or from extensions.

## The Problem

SystemPrompt applications need ongoing maintenance. Content needs to be published, old sessions need cleanup, and databases need optimization. Without automation, these tasks would require manual intervention or external cron systems.

The scheduler service solves this by providing built-in job scheduling. Define jobs in configuration, and the scheduler runs them at the specified times. Jobs can perform any operation - from simple cleanup to complex content processing.

## How Scheduler Works

The scheduler runs as part of the SystemPrompt application. At startup, it reads job definitions and sets up cron triggers. When a trigger fires, the scheduler executes the job function from the specified extension.

Jobs run asynchronously and do not block the main application. Each job has its own context and can access databases, APIs, and other services. Failed jobs are logged but do not affect other jobs.

## Configuration

Configure jobs in `services/scheduler/config.yaml`:

<details>
<summary>Scheduler configuration</summary>

```yaml
# services/scheduler/config.yaml
scheduler:
  enabled: true
  jobs:
    - name: cleanup_anonymous_users
      extension: core
      job: cleanup_anonymous_users
      schedule: "0 0 3 * * *"
      enabled: true

    - name: cleanup_empty_contexts
      extension: core
      job: cleanup_empty_contexts
      schedule: "0 0 * * * *"
      enabled: true

    - name: publish_content
      extension: core
      job: publish_content
      schedule: "0 */30 * * * *"
      enabled: true

    - name: content_ingestion
      extension: blog
      job: content_ingestion
      schedule: "0 0 * * * *"
      enabled: true
```

</details>

## Job Definition

Each job requires these fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique job identifier |
| `extension` | string | Extension providing the job function |
| `job` | string | Job function name |
| `schedule` | string | Cron expression |
| `enabled` | boolean | Whether job is active |

## Cron Schedule Format

The scheduler uses 6-field cron expressions:

```
┌──────────── second (0-59)
│ ┌────────── minute (0-59)
│ │ ┌──────── hour (0-23)
│ │ │ ┌────── day of month (1-31)
│ │ │ │ ┌──── month (1-12)
│ │ │ │ │ ┌── day of week (0-6, 0=Sunday)
│ │ │ │ │ │
* * * * * *
```

Common schedules:
- `0 0 * * * *` - Every hour at minute 0
- `0 */30 * * * *` - Every 30 minutes
- `0 0 3 * * *` - Daily at 3:00 AM
- `0 0 0 * * 0` - Weekly on Sunday at midnight
- `0 0 0 1 * *` - Monthly on the 1st

## Core Jobs

These jobs are provided by `systemprompt-core`:

| Job | Default Schedule | Description |
|-----|------------------|-------------|
| `cleanup_anonymous_users` | Daily 3 AM | Remove old anonymous user accounts |
| `cleanup_empty_contexts` | Hourly | Clean up empty conversation contexts |
| `cleanup_inactive_sessions` | Hourly | Remove inactive sessions |
| `database_cleanup` | Daily 4 AM | General database maintenance |
| `publish_content` | Every 30 min | Sync content files to database |

These jobs keep the system running smoothly without manual intervention.

## Extension Jobs

Extensions can register custom jobs. The extension provides the job function, and you configure it in the scheduler:

```yaml
- name: content_ingestion
  extension: blog       # Extension providing the job
  job: content_ingestion
  schedule: "0 0 * * * *"
  enabled: true
```

The extension must export a job function with the matching name.

## Managing Jobs

Use the CLI to manage scheduled jobs:

```bash
# List all jobs
systemprompt infra jobs list

# Run a job manually
systemprompt infra jobs run cleanup_anonymous_users

# View job history
systemprompt infra jobs history
systemprompt infra jobs history --job publish_content

# Enable/disable jobs
systemprompt infra jobs disable content_ingestion
systemprompt infra jobs enable content_ingestion
```

Running a job manually does not affect its schedule. The next scheduled run will still occur at the configured time.

## Service Relationships

The scheduler connects to:

- **Content service** - Runs `publish_content` to sync content files
- **Config service** - Included through the aggregation pattern
- **Extensions** - Executes job functions provided by extensions

The scheduler is independent of agents and MCP servers. It runs maintenance tasks that support all other services.

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt infra jobs list` | List available jobs |
| `systemprompt infra jobs show <name>` | Show detailed information about a job |
| `systemprompt infra jobs run <name>` | Run a scheduled job manually |
| `systemprompt infra jobs history` | View job execution history |
| `systemprompt infra jobs enable <name>` | Enable a job |
| `systemprompt infra jobs disable <name>` | Disable a job |
| `systemprompt infra jobs cleanup-sessions` | Clean up inactive sessions |
| `systemprompt infra jobs log-cleanup` | Clean up old log entries |

See `systemprompt infra jobs <command> --help` for detailed options.

## Troubleshooting

**Job not running** -- Check that the job is enabled and the cron expression is valid. Verify the extension is loaded and the job function exists.

**Job failing silently** -- Check application logs for error messages. Jobs log their execution and any errors.

**Schedule not working as expected** -- Remember that cron uses 6 fields (including seconds). Verify the expression with a cron calculator.