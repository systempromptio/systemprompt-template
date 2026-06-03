# Analytics Dashboards

Read the workspace at a glance: spend, request volume, agent performance, tool reliability, sessions, and
conversations - all rolled up over a time window. This is the fleet view; for drilling into one request use
`inspect_ai_requests`, and for one conversation use `inspect_conversation`.

## When to Use

Use this skill to answer "how is the whole system doing?": total cost and where it goes, request latency and error
rate, which agents are busiest and most reliable, which tools fail, and how many sessions are live.

## Time windows

Every analytics command takes `--since` (default `24h`, e.g. `1h`, `7d`, `30d`) and most take `--until`. Each
topic needs a verb, and the verbs differ per topic - not every topic has every verb. The real set is:

| Topic | Verbs |
|-------|-------|
| `costs` | `summary`, `trends`, `breakdown --by {model,agent,provider}` |
| `requests` | `stats`, `list`, `trends`, `models` |
| `agents` | `stats`, `list`, `trends`, `show <agent>` |
| `tools` | `stats`, `list`, `trends`, `show <tool>` |
| `sessions` | `stats`, `trends`, `live` |
| `conversations` | `stats`, `trends`, `list` |
| `content` | `stats`, `top`, `trends` |
| `traffic` | `sources`, `geo`, `devices`, `bots` |

`overview` takes no verb. When unsure, `systemprompt analytics <topic> --help` is authoritative.

## Reading attribution (read this before reporting agent numbers)

The fleet runs two distinct traffic paths, and they attribute differently:

- **Gateway inference** - every `/v1/messages` hit (Cowork, any Anthropic-SDK client, the playground). These
  land in `ai_requests` and drive `costs` and `requests`, but they carry **no agent task**, so they show as
  `unattributed` under `costs breakdown --by agent` and produce **no rows** in `agents list` / `agents stats`.
- **Agent tasks** - spawned agent runs and their MCP tool calls. These are what populate `agents` and `tools`.

So `costs breakdown --by agent` reading 100% `unattributed`, or `agents list` saying "No agents found", is the
**expected** shape of a workspace whose traffic is gateway inference rather than spawned agent tasks. It is
**not** a tracking failure or a missing-data bug - do not report it as one. Report it as "spend is gateway
inference, not attributed to named agents", and pivot to `costs breakdown --by model` / `--by provider` and
`requests models` to show where that spend actually goes. Only call attribution genuinely broken if agent tasks
*were* run in the window and still don't appear.

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

### Content and traffic

```bash
systemprompt analytics content stats          # content engagement
systemprompt analytics content top            # top performing content
systemprompt analytics traffic sources        # referrers / channels
systemprompt analytics traffic geo            # geographic distribution
systemprompt analytics traffic bots           # bot vs human traffic
```

### Typical workflow

1. `analytics overview` - take the temperature.
2. `analytics costs breakdown --by agent` - see where spend concentrates. If it reads 100% `unattributed`,
   that is gateway inference (see "Reading attribution"); pivot to `--by model` / `--by provider`.
3. `analytics requests models` - show the routing mix and where token spend lands.
4. `analytics agents list` / `analytics tools stats` - find the slow or failing actors (empty `agents` just
   means no agent tasks ran in the window, not a fault).
5. Hand a specific request or conversation to `inspect_ai_requests` / `inspect_conversation` for the deep dive.
