---
title: "Scheduler Jobs"
description: "Configure and manage scheduled jobs with cron expressions for background automation."
author: "SystemPrompt"
slug: "domain-scheduler-jobs"
keywords: "scheduler, jobs, cron, automation, background"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Scheduler Jobs

Job scheduling and automation. Config: `services/scheduler/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_scheduler-jobs" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Configure Jobs

Edit `services/scheduler/config.yaml`:

```yaml
scheduler:
  enabled: true
  timezone: "UTC"
  jobs:
    - name: publish_pipeline
      schedule: "0 */6 * * *"
      enabled: true
      description: "Publish content changes"
      timeout_seconds: 300
    - name: cleanup_sessions
      schedule: "0 0 * * *"
      enabled: true
      description: "Clean up expired sessions"
      timeout_seconds: 60
    - name: cleanup_anonymous_users
      schedule: "0 2 * * *"
      enabled: true
      description: "Remove anonymous users older than 30 days"
      timeout_seconds: 120
    - name: database_cleanup
      schedule: "0 3 * * 0"
      enabled: true
      description: "Database maintenance"
      timeout_seconds: 600
    - name: analytics_aggregate
      schedule: "0 */1 * * *"
      enabled: true
      description: "Aggregate analytics data"
      timeout_seconds: 120
```

---

## Cron Format

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6, Sunday = 0)
* * * * *
```

| Expression | Description |
|------------|-------------|
| `* * * * *` | Every minute |
| `0 * * * *` | Every hour |
| `0 0 * * *` | Daily at midnight |
| `0 */6 * * *` | Every 6 hours |
| `0 0 * * 0` | Weekly on Sunday |
| `0 0 1 * *` | Monthly on 1st |

---

## Monitor Jobs

{ "command": "infra jobs list" }
{ "command": "infra jobs status publish_pipeline" }
{ "command": "infra jobs history publish_pipeline" }
{ "command": "infra logs --context scheduler --limit 50" }

---

## Run Manually

{ "command": "infra jobs run publish_pipeline" }
{ "command": "infra jobs run publish_pipeline --verbose" }

---

## Create Custom Job

```yaml
jobs:
  - name: custom_sync
    schedule: "0 */2 * * *"
    enabled: true
    description: "Custom sync operation"
    timeout_seconds: 180
    config:
      source: "external_api"
      batch_size: 100
```

{ "command": "infra jobs list" }
{ "command": "infra jobs run custom_sync" }

---

## Enable/Disable Jobs

Disable:

```yaml
jobs:
  - name: publish_pipeline
    enabled: false
```

Enable:

```yaml
jobs:
  - name: publish_pipeline
    enabled: true
```

{ "command": "infra jobs list" }

---

## Common Schedules

| Use Case | Expression |
|----------|------------|
| Every 5 minutes | `*/5 * * * *` |
| Every 15 minutes | `*/15 * * * *` |
| Every hour | `0 * * * *` |
| Every 4 hours | `0 */4 * * *` |
| Daily midnight | `0 0 * * *` |
| Daily 3 AM | `0 3 * * *` |
| Weekly Sunday | `0 0 * * 0` |
| Monthly 1st | `0 0 1 * *` |

---

## Built-in Jobs

publish_pipeline:

```yaml
- name: publish_pipeline
  schedule: "0 */6 * * *"
  description: "Publish content changes"
```

cleanup_sessions:

```yaml
- name: cleanup_sessions
  schedule: "0 0 * * *"
  description: "Clean up expired sessions"
```

database_cleanup:

```yaml
- name: database_cleanup
  schedule: "0 3 * * 0"
  description: "Database maintenance"
```

---

## Configuration Reference

```yaml
jobs:
  - name: job_name
    schedule: "0 * * * *"
    enabled: true
    description: "Description"
    timeout_seconds: 300
    config:
      key: value
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier |
| `schedule` | Yes | Cron expression |
| `enabled` | Yes | Active state |
| `description` | No | What job does |
| `timeout_seconds` | No | Max execution time |
| `config` | No | Job-specific settings |

---

## Startup Jobs

Jobs can run immediately when the scheduler starts. This requires:

1. **Code**: Job implements `run_on_startup() -> true`
2. **Config**: Job is listed here with `enabled: true`

Both conditions must be met. The config acts as an "allow list" - even if code says run on startup, the job won't run unless configured.

Example startup job:

```yaml
jobs:
  - name: publish_pipeline
    extension: web
    job: publish_pipeline
    schedule: "0 */15 * * * *"
    enabled: true  # Required for startup
```

The `publish_pipeline` job has `run_on_startup: true` in code, so it runs:
1. Once immediately when scheduler starts
2. Every 15 minutes thereafter

-> See [Creating Jobs](../../build/library/job.md) for implementing `run_on_startup()`.

---

## Troubleshooting

- Job not running: `{ "command": "infra jobs list" }`, check `enabled: true`
- Job fails: `{ "command": "infra logs --context scheduler --level error" }`, `{ "command": "infra jobs history <job_name>" }`
- Job times out: Increase `timeout_seconds` in config

---

## Quick Reference

| Task | Command |
|------|---------|
| List | `infra jobs list` |
| Status | `infra jobs status <name>` |
| Run | `infra jobs run <name>` |
| History | `infra jobs history <name>` |
| Logs | `infra logs --context scheduler` |

---

## Related

-> See [Scheduler Service](/documentation/services/scheduler)