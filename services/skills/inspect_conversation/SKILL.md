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

### Typical workflow

1. `analytics sessions live` - grab the active `session_id`.
2. `analytics conversations list` - map it to a `context_id` and `core contexts show <context_id>`.
3. `infra db query` over `ai_requests WHERE session_id = …` - list the requests.
4. `infra logs audit <request_id> --full` - reconstruct each request's messages, tool calls, and cost.
5. `infra db query` over `governance_decisions WHERE session_id = …` - show what governance allowed or denied.

The result is the complete structured record of one conversation, assembled from the live database.
