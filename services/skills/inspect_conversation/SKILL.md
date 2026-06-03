# Inspect Conversation

Show the structured data behind a conversation that is happening right now. When a client like Claude Desktop talks
to the gateway, every message, AI request, tool call, governance decision, and cost lands in the database keyed to one
session. This skill reconstructs that whole picture from a single starting id.

## When to Use

Use this skill when someone asks "what is actually being stored about this conversation?" or "show me the data trail
behind what I just did." It is the self-referential demo: point it at the active session and it returns the messages,
the provider requests they triggered, the tools those requests called, the governance rulings, and the spend.

## The id chain

One conversation threads through five ids. Each step below walks one link:

```
session_id  ->  context_id  ->  task_id  ->  trace_id  ->  request_id
(gateway)       (conversation)  (a turn)     (execution)   (one /v1/messages hit)
```

No single command returns all of it; compose the steps. Where there is no dedicated command for a link, use
`infra db query` (read-only SQL).

### Two anchors: pick the right starting id

The conversation type determines which id anchors the reconstruction:

- **Gateway-client conversations** (Claude Desktop, Cowork, any Anthropic-SDK client hitting `/v1/messages`)
  are keyed on `session_id`. Start from `session_id` and walk to `ai_requests` directly (the path in steps
  3-5 below).
- **A2A / internal agent runs** (an agent driven via `admin agents message … --context-id`) do **not** share
  the gateway `session_id`. They are grouped by `context_id`, and the `agent_tasks` table is the bridge:
  `context_id` -> `agent_tasks` (one row per turn) -> `trace_id` / `request_id`. Use the `context_id` path in
  step 6 below to reconstruct these.

Both are first-class; reach for `session_id` for gateway clients and `context_id` for agent runs.

## How to Use

### 1. Find the live conversation

```bash
systemprompt analytics sessions live                  # active sessions: session_id, request_count, last activity
systemprompt analytics conversations list --limit 5   # recent conversations: context_id, name, message/task counts
systemprompt core contexts list                       # contexts with an is_active flag
```

Pick the session you are interested in (most recent / highest `request_count`), and note its `session_id` and the
matching `context_id`.

### 2. Open the conversation

```bash
systemprompt core contexts show <context_id>          # name, task count, message count, timestamps
```

### 3. Pull the requests this conversation made

Every turn fires one or more `/v1/messages` requests. List them for the session, then reconstruct each:

```bash
# requests tied to this session (no flag for session on `request list`, so query directly):
systemprompt infra db query "SELECT id, model, status, input_tokens, output_tokens, cost_microdollars, created_at FROM ai_requests WHERE session_id = '<session_id>' ORDER BY created_at DESC"

# full reconstruction of one request - identity, prompt, response, tool calls, cost:
systemprompt infra logs audit <request_id> --full
```

### 4. Follow the tool calls

If a request spawned MCP tool calls, follow them by trace:

```bash
systemprompt infra logs trace show <trace_id> --all
```

### 5. Show the governance rulings and cost for this conversation

```bash
systemprompt infra db query "SELECT decision, tool_name, policy, reason, created_at FROM governance_decisions WHERE session_id = '<session_id>' ORDER BY created_at DESC"
systemprompt analytics costs summary --since 24h      # spend over the window
```

### 6. Reconstruct an A2A agent run via `context_id`

Internal agent runs are not gateway sessions, so steps 3-5's `session_id` lookups return nothing for them.
Anchor on `context_id` instead and use `agent_tasks` as the bridge — one row per turn, carrying `task_id`,
`session_id`, `trace_id`, `agent_name`, `user_id`, and the `started_at`/`completed_at` window:

```bash
# the agent's turns for this conversation:
systemprompt infra db query "SELECT task_id, agent_name, user_id, session_id, trace_id, started_at, completed_at FROM agent_tasks WHERE context_id = '<context_id>' ORDER BY started_at"

# per turn, reconstruct execution and requests:
systemprompt infra logs trace show <trace_id> --all
systemprompt infra logs audit <request_id> --full
```

The agent's own tool calls are governed under a **separate MCP session**, so `governance_decisions` rows for
the run are not keyed by the conversation `session_id`. Join them by `user_id` within the turn's time window:

```bash
systemprompt infra db query "SELECT decision, tool_name, policy, reason, created_at FROM governance_decisions WHERE user_id = '<user_id>' AND created_at BETWEEN '<started_at>' AND '<completed_at>' ORDER BY created_at"
```

> Residual nit (not blocking): `governance_decisions` carries no `context_id`/`task_id` column, so the
> agent-tool join is `user_id` + time window rather than a direct key. Adding `context_id`/`task_id` to
> `governance_decisions` would make this an exact join — a possible small core enhancement, logged but not
> done here.

### Typical workflow

1. `analytics sessions live` - grab the active `session_id`.
2. `analytics conversations list` - map it to a `context_id` and `core contexts show <context_id>`.
3. `infra db query` over `ai_requests WHERE session_id = …` - list the requests.
4. `infra logs audit <request_id> --full` - reconstruct each request's messages, tool calls, and cost.
5. `infra db query` over `governance_decisions WHERE session_id = …` - show what governance allowed or denied.

The result is the complete structured record of one conversation, assembled from the live database.
