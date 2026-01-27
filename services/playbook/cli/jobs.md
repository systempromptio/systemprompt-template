---
title: "Jobs & Scheduling Playbook"
description: "Run and manage background jobs."
keywords:
  - jobs
  - scheduler
  - cron
  - background
---

# Jobs & Scheduling Playbook

Run and manage background jobs.

> **Help**: `{ "command": "infra jobs" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## List Jobs

```json
// MCP: systemprompt
{ "command": "infra jobs list" }
```

Shows name, schedule (cron), enabled status, and last run time.

---

## Show Job Details

```json
// MCP: systemprompt
{ "command": "infra jobs show cleanup-sessions" }
{ "command": "infra jobs show log-cleanup" }
```

---

## Run Job Manually

```json
// MCP: systemprompt
{ "command": "infra jobs run cleanup-sessions" }
{ "command": "infra jobs run log-cleanup" }
```

---

## View Job History

```json
// MCP: systemprompt
{ "command": "infra jobs history" }
{ "command": "infra jobs history --limit 20" }
{ "command": "infra jobs history --job cleanup-sessions" }
```

---

## Enable/Disable Jobs

```json
// MCP: systemprompt
{ "command": "infra jobs enable cleanup-sessions" }
{ "command": "infra jobs disable cleanup-sessions" }
```

---

## Built-in Jobs

```json
// MCP: systemprompt
{ "command": "infra jobs cleanup-sessions" }
{ "command": "infra jobs cleanup-sessions --hours 24" }
{ "command": "infra jobs cleanup-sessions --hours 48" }
{ "command": "infra jobs log-cleanup" }
{ "command": "infra jobs log-cleanup --days 30" }
```

---

## Troubleshooting

**Job not running** -- check if enabled with `infra jobs show <job-name>`, then `infra jobs enable <job-name>`.

**Job failing** -- check logs:
```json
// MCP: systemprompt
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
