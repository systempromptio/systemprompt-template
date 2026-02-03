---
title: "Jobs & Scheduling Playbook"
description: "Run and manage background jobs."
author: "SystemPrompt"
slug: "cli-jobs"
keywords: "jobs, scheduler, cron, background"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Jobs & Scheduling Playbook

Run and manage background jobs.

---

## List Jobs

```json
{ "command": "infra jobs list" }
```

Shows name, schedule (cron), enabled status, and last run time.

---

## Show Job Details

```json
{ "command": "infra jobs show cleanup-sessions" }
{ "command": "infra jobs show log-cleanup" }
```

---

## Run Job Manually

```json
{ "command": "infra jobs run cleanup-sessions" }
{ "command": "infra jobs run log-cleanup" }
```

---

## View Job History

```json
{ "command": "infra jobs history" }
{ "command": "infra jobs history --limit 20" }
{ "command": "infra jobs history --job cleanup-sessions" }
```

---

## Enable/Disable Jobs

```json
{ "command": "infra jobs enable cleanup-sessions" }
{ "command": "infra jobs disable cleanup-sessions" }
```

---

## Built-in Jobs

```json
{ "command": "infra jobs cleanup-sessions" }
{ "command": "infra jobs cleanup-sessions --hours 24" }
{ "command": "infra jobs cleanup-sessions --hours 48" }
{ "command": "infra jobs log-cleanup" }
{ "command": "infra jobs log-cleanup --days 30" }
```

---

## Troubleshooting

**Job not running** -- check if enabled with `infra jobs show <job-name>`, then `infra jobs enable <job-name>`.

**Startup job not running** -- Job needs BOTH `run_on_startup() = true` in code AND entry in `services/scheduler/config.yaml`. See [Scheduler Jobs](../domain/scheduler/jobs.md).

**Job failing** -- check logs:
```json
{ "command": "infra logs view --level error --since 1h" }
{ "command": "infra logs search \"job failed\"" }
```

---

## Quick Reference

| Task | Command |
|------|---------|
| List jobs | `infra jobs list` |
| Show job | `infra jobs show <name>` |
| Run job | `infra jobs run <name>` |
| Job history | `infra jobs history` |
| Enable job | `infra jobs enable <name>` |
| Disable job | `infra jobs disable <name>` |
| Cleanup sessions | `infra jobs cleanup-sessions --hours 24` |
| Cleanup logs | `infra jobs log-cleanup --days 30` |