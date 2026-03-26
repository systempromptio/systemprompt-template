---
name: "AI Usage Report"
description: "Generate AI usage reports for leadership: team activity, costs, tool adoption, and session analytics"
---

# AI Usage Report

You generate AI usage reports for leadership stakeholders using the systemprompt CLI. Reports cover team activity, cost trends, tool adoption, and per-user breakdowns.

All commands support `--since` (e.g. `1h`, `24h`, `7d`, `30d`), `--until`, and `--export <file.csv>`.

## Team Overview

| Command | Purpose |
|---------|---------|
| `systemprompt analytics overview` | High-level dashboard with key metrics |
| `systemprompt analytics overview --since 7d` | Weekly overview |
| `systemprompt analytics overview --since 30d` | Monthly overview |

## Per-User & Session Breakdown

| Command | Purpose |
|---------|---------|
| `systemprompt analytics sessions stats` | Session metrics and duration stats |
| `systemprompt analytics sessions list --since 7d` | List sessions for the past week |
| `systemprompt analytics sessions trends --since 30d` | Session trends over time |

## Cost Summary

| Command | Purpose |
|---------|---------|
| `systemprompt analytics costs summary` | Cost summary across all providers |
| `systemprompt analytics costs summary --since 30d` | Monthly cost summary |
| `systemprompt analytics costs breakdown --by model` | Cost breakdown by model |
| `systemprompt analytics costs breakdown --by agent` | Cost breakdown by agent |
| `systemprompt analytics costs trends --since 30d` | Cost trends over time |

## Tool Adoption

| Command | Purpose |
|---------|---------|
| `systemprompt analytics tools stats` | Aggregate tool statistics |
| `systemprompt analytics tools list --since 7d` | Tools used in the past week |
| `systemprompt analytics tools list --sort-by execution-count` | Most-used tools |
| `systemprompt analytics tools trends --since 30d` | Tool adoption trends |

## Agent Performance

| Command | Purpose |
|---------|---------|
| `systemprompt analytics agents stats` | Aggregate agent statistics |
| `systemprompt analytics agents list --sort-by cost` | Agents ranked by cost |
| `systemprompt analytics agents list --sort-by success-rate` | Agents ranked by success rate |
| `systemprompt analytics agents trends --since 30d` | Agent usage trends |

## Standard Report Workflow

1. **Start with the overview** to get key metrics at a glance
2. **Check costs** to understand spending trends and identify outliers
3. **Review sessions** for per-user activity and engagement
4. **Assess tool adoption** to see which AI capabilities the team is using
5. **Export data** using `--export report.csv` for presentations or further analysis

## Weekly Leadership Report

```bash
systemprompt analytics overview --since 7d
systemprompt analytics costs summary --since 7d
systemprompt analytics sessions stats --since 7d
systemprompt analytics tools stats --since 7d
systemprompt analytics agents stats --since 7d
```

## Monthly Leadership Report

```bash
systemprompt analytics overview --since 30d
systemprompt analytics costs summary --since 30d
systemprompt analytics costs breakdown --by model --since 30d
systemprompt analytics sessions trends --since 30d
systemprompt analytics tools trends --since 30d
systemprompt analytics agents list --sort-by cost --since 30d
```

## Important Notes

- Time ranges use shorthand: `1h`, `24h`, `7d`, `30d`
- Cost data may lag behind real-time by a few minutes
- Use `--export <file.csv>` to export any analytics result for presentations
- Use `--help` on any subcommand for full flag reference
- Combine multiple exports to build a comprehensive leadership dashboard
