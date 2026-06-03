# Analytics Dashboards

Read the workspace at a glance: spend, request volume, agent performance, tool reliability, sessions, and
conversations - all rolled up over a time window. This is the fleet view; for drilling into one request use
`inspect_ai_requests`, and for one conversation use `inspect_conversation`.

## When to Use

Use this skill to answer "how is the whole system doing?": total cost and where it goes, request latency and error
rate, which agents are busiest and most reliable, which tools fail, and how many sessions are live.

## Time windows

Every analytics command takes `--since` (default `24h`, e.g. `1h`, `7d`, `30d`) and most take `--until`. Each topic
needs a verb: `summary`, `stats`, `list`, `trends`, `breakdown`, `models`, or `show`.

## How to Use

### One-screen overview

```bash
systemprompt analytics overview               # all domains, period-over-period
systemprompt analytics overview --since 7d
```

### Cost

```bash
systemprompt analytics costs summary          # total spend, requests, tokens, avg cost/request
systemprompt analytics costs breakdown --by model     # or: --by agent, --by provider
systemprompt analytics costs trends --since 7d
```

### Requests and routing

```bash
systemprompt analytics requests stats         # volume, tokens, latency, cache hit rate
systemprompt analytics requests models        # per-model breakdown (shows routing mix)
systemprompt analytics requests list --limit 20
```

### Agents

```bash
systemprompt analytics agents stats
systemprompt analytics agents list            # per-agent task count, success rate, cost
systemprompt analytics agents show developer_agent
systemprompt analytics agents trends --since 7d
```

### Tools

```bash
systemprompt analytics tools stats            # executions, success rate, p95 latency
systemprompt analytics tools list
systemprompt analytics tools show <tool-name>
```

### Sessions and conversations

```bash
systemprompt analytics sessions live          # active sessions right now
systemprompt analytics sessions stats
systemprompt analytics conversations list     # recent conversations: context_id, name, counts
```

### Typical workflow

1. `analytics overview` - take the temperature.
2. `analytics costs breakdown --by agent` - see where spend concentrates.
3. `analytics agents list` / `analytics tools stats` - find the slow or failing actors.
4. Hand a specific request or conversation to `inspect_ai_requests` / `inspect_conversation` for the deep dive.
