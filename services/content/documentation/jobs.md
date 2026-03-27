---
title: "Jobs"
description: "Monitor scheduled background jobs that keep your systemprompt.io platform running. View job schedules, statuses, run history, and errors from the admin dashboard."
author: "systemprompt.io"
slug: "jobs"
keywords: "jobs, background tasks, scheduling, cron, monitoring, maintenance"
kind: "guide"
public: true
tags: ["jobs", "admin", "monitoring"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "View all scheduled background jobs"
  - "Understand job statuses and schedules"
  - "Identify failed jobs and their error messages"
  - "Monitor job run history and next execution times"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Events"
    url: "/documentation/events"
  - title: "Users"
    url: "/documentation/users"
---

# Jobs

**TL;DR:** The Jobs page shows all scheduled background jobs running on the platform. Each job has a schedule (cron expression), enabled/disabled status, last run time, success/failure status, and next scheduled run. Use this page to monitor platform health and identify jobs that are failing. Admin access is required.

## Access Control

The Jobs page (`/admin/jobs/`) is **admin-only**. Non-admin users receive a 403 Forbidden response.

## What You'll See

### Jobs Table

The main view is a data table listing all registered background jobs:

| Column | Description |
|--------|-------------|
| **Job** | The job name identifier |
| **Schedule** | Cron expression shown as inline code (e.g., `0 */6 * * *`) |
| **Enabled** | Green "Enabled" or gray "Disabled" badge |
| **Last Run** | Relative timestamp of the most recent execution (or "Never") |
| **Status** | Result of the last run — see status badges below |
| **Next Run** | Relative timestamp of the next scheduled execution |
| **Runs** | Total number of times the job has executed |

### Job Statuses

| Status | Badge | Description |
|--------|-------|-------------|
| `success` | Green "Success" | The last run completed without errors |
| `failed` | Red "Failed" | The last run encountered an error (hover for error message) |
| `running` | Blue "Running" | The job is currently executing |
| No status | Gray "Never run" | The job has not yet executed |

When a job has a `failed` status and a `last_error` value, the error message is available as a tooltip — hover over the status badge to see it.

### Search

A search bar in the toolbar lets you filter jobs by name. The filter is client-side with a 200ms debounce, matching against the job name.

## Understanding Job Schedules

Job schedules use cron syntax. Common patterns:

| Schedule | Meaning |
|----------|---------|
| `* * * * *` | Every minute |
| `*/5 * * * *` | Every 5 minutes |
| `0 * * * *` | Every hour |
| `0 */6 * * *` | Every 6 hours |
| `0 0 * * *` | Daily at midnight |
| `0 0 * * 0` | Weekly on Sunday |

## Monitoring Jobs

### Healthy State

A healthy platform has all enabled jobs showing green "Success" badges with recent "Last Run" timestamps and upcoming "Next Run" times.

### Warning Signs

Watch for these indicators:

- **Failed status** — A job failed on its last run. Hover the badge to see the error.
- **High run count with recent failure** — A job that has been running successfully but recently started failing may indicate an environment change.
- **"Never" last run on an enabled job** — The job may not be scheduling correctly.
- **Missing next run time** — The job scheduler may not be running.
