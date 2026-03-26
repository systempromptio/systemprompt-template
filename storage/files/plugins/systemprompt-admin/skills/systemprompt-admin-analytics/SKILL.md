---
name: "Analytics"
description: "View traffic, costs, agent stats, and bot detection via the systemprompt CLI"
---

# Analytics

You view platform analytics including traffic, costs, agent performance, and bot detection using the systemprompt CLI. All operations go through the `analytics` domain.

All analytics commands support `--since` (e.g. `1h`, `24h`, `7d`, `30d`), `--until`, and `--export <file.csv>`.

## Dashboard & Overview

| Command | Purpose |
|---------|---------|
| `systemprompt analytics overview` | Overview dashboard with key metrics |
| `systemprompt analytics overview --since 7d` | Dashboard for the last 7 days |

## Conversation Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics conversations stats` | Conversation statistics |
| `systemprompt analytics conversations trends --since 7d` | Conversation trends over time |
| `systemprompt analytics conversations list --limit 20` | List recent conversations |

## Session Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics sessions stats` | Session metrics and duration stats |
| `systemprompt analytics sessions trends --since 7d` | Session trends over time |
| `systemprompt analytics sessions live` | Real-time active sessions |

## Content Performance

| Command | Purpose |
|---------|---------|
| `systemprompt analytics content stats` | Content engagement statistics |
| `systemprompt analytics content top --limit 10` | Top performing content |
| `systemprompt analytics content trends --since 7d` | Content trends over time |

## Cost Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics costs summary` | Cost summary across all providers |
| `systemprompt analytics costs breakdown --by model` | Cost breakdown by model |
| `systemprompt analytics costs breakdown --by agent` | Cost breakdown by agent |
| `systemprompt analytics costs breakdown --by provider` | Cost breakdown by provider |
| `systemprompt analytics costs trends --since 7d` | Cost trends over time |

## Agent Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics agents stats` | Aggregate agent statistics |
| `systemprompt analytics agents list` | List agents with metrics |
| `systemprompt analytics agents list --sort-by cost` | Sort by cost (task-count, success-rate, cost, last-active) |
| `systemprompt analytics agents show <agent>` | Deep dive on a specific agent |
| `systemprompt analytics agents trends --since 7d` | Agent usage trends |

## Tool Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics tools stats` | Aggregate tool statistics |
| `systemprompt analytics tools list` | List tools with metrics |
| `systemprompt analytics tools list --sort-by execution-count` | Sort by metric (execution-count, success-rate, avg-time) |
| `systemprompt analytics tools show <tool>` | Deep dive on a specific tool |
| `systemprompt analytics tools trends --since 7d` | Tool usage trends |

## AI Request Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics requests stats` | AI request volume and latency |
| `systemprompt analytics requests list --since 24h` | List recent AI requests |
| `systemprompt analytics requests trends --since 7d` | Request trends over time |
| `systemprompt analytics requests models` | Model usage breakdown |

## Traffic Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics traffic sources` | Traffic source breakdown |
| `systemprompt analytics traffic geo` | Geographic distribution |
| `systemprompt analytics traffic devices` | Device and browser breakdown |
| `systemprompt analytics traffic bots` | Bot traffic analysis |

## Standard Workflow

1. **Start with the overview** to get a high-level dashboard
2. **Drill down** into specific areas (costs, agents, traffic) as needed
3. **Filter by time** using `--since` to focus on relevant periods
4. **Export data** using `--export report.csv` for further analysis

## Common Tasks

### Daily Health Check

```bash
systemprompt analytics overview
systemprompt analytics costs summary
systemprompt analytics traffic bots
```

### Investigate High Costs

```bash
systemprompt analytics costs breakdown --by model
systemprompt analytics costs breakdown --by agent
systemprompt analytics costs trends --since 7d
```

### Check Agent Performance

```bash
systemprompt analytics agents stats
systemprompt analytics agents list --sort-by success-rate
systemprompt analytics tools stats
```

### Monitor Traffic

```bash
systemprompt analytics traffic sources
systemprompt analytics traffic geo
systemprompt analytics sessions live
```

## Important Notes

- Time ranges use shorthand: `1h`, `24h`, `7d`, `30d`
- Cost data may lag behind real-time by a few minutes
- Bot detection results are heuristic-based -- review before taking action
- Use `--export <file.csv>` to export any analytics result
- Use `--help` on any subcommand for full flag reference
