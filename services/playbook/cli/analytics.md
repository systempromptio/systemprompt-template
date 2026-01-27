---
title: "Analytics Playbook"
description: "View metrics, performance data, and cost analysis."
keywords:
  - analytics
  - metrics
  - costs
  - performance
---

# Analytics Playbook

View metrics, performance data, and cost analysis.

> **Help**: `{ "command": "analytics" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Dashboard Overview

```json
// MCP: systemprompt
{ "command": "analytics overview" }
```

---

## Agent Analytics

```json
// MCP: systemprompt
{ "command": "analytics agents stats" }
{ "command": "analytics agents trends --days 7" }
{ "command": "analytics agents show <name>" }
{ "command": "analytics agents list" }
```

---

## Cost Analytics

```json
// MCP: systemprompt
{ "command": "analytics costs summary" }
{ "command": "analytics costs summary --days 30" }
{ "command": "analytics costs trends" }
{ "command": "analytics costs breakdown --by agent" }
{ "command": "analytics costs breakdown --by model" }
```

---

## Content Analytics

```json
// MCP: systemprompt
{ "command": "analytics content stats" }
{ "command": "analytics content trends --days 30" }
{ "command": "analytics content popular --limit 20" }
```

---

## Traffic Analytics

```json
// MCP: systemprompt
{ "command": "analytics traffic sources" }
{ "command": "analytics traffic geo" }
{ "command": "analytics traffic devices" }
{ "command": "analytics traffic bots" }
```

---

## Session Analytics

```json
// MCP: systemprompt
{ "command": "analytics sessions stats" }
{ "command": "analytics sessions trends" }
{ "command": "analytics sessions live" }
```

---

## Conversation Analytics

```json
// MCP: systemprompt
{ "command": "analytics conversations stats" }
{ "command": "analytics conversations trends" }
{ "command": "analytics conversations list" }
```

---

## Tool Usage Analytics

```json
// MCP: systemprompt
{ "command": "analytics tools stats" }
{ "command": "analytics tools trends" }
{ "command": "analytics tools show <tool-name>" }
```

---

## AI Request Analytics

```json
// MCP: systemprompt
{ "command": "analytics requests stats" }
{ "command": "analytics requests list" }
{ "command": "analytics requests list --limit 50 --model claude" }
{ "command": "analytics requests trends" }
{ "command": "analytics requests models" }
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Overview | `analytics overview` |
| Agent stats | `analytics agents stats` |
| Agent trends | `analytics agents trends --days 7` |
| Cost summary | `analytics costs summary` |
| Cost breakdown | `analytics costs breakdown` |
| Content stats | `analytics content stats` |
| Popular content | `analytics content popular` |
| Traffic sources | `analytics traffic sources` |
| Session stats | `analytics sessions stats` |
| Tool usage | `analytics tools stats` |
| Request stats | `analytics requests stats` |
| List requests | `analytics requests list` |
