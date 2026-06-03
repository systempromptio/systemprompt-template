# Inspect AI Requests

Follow one inference request through the AI gateway: from the client's `/v1/messages` call, through model routing and
the governance pipeline, to the provider and back. Use it to debug a single request - routing, denials, latency, cost,
or the tool calls it spawned.

## When to Use

Use this skill when a specific request needs explaining: which provider it routed to, whether governance allowed it,
what the provider returned, what it cost, and what tools it triggered. For fleet-wide rollups and dashboards use
`analytics_dashboards` instead; this skill is the per-request microscope.

## The gateway path

Clients call `/v1/messages` on the gateway. The gateway routes by model pattern to a provider (see the
`gateway.routes` block in the active profile - e.g. `claude-*` -> anthropic, `gpt-*` -> openai), runs the synchronous
governance pipeline, forwards to the provider, and audits the result. Every hit is one row carrying `user_id`,
`tenant_id`, `session_id`, and `trace_id`, so a single id reconstructs the whole chain. `infra logs request` is that
gateway message log.

## How to Use

### 1. List recent gateway requests

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --model claude
systemprompt infra logs request list --since 24h --provider anthropic
```

Each row is one `/v1/messages` hit: provider, model, token counts, cost, latency, status. (Filters are `--since`,
`--model`, `--provider`, `--limit`. To filter by failure status, use the trace view in step 3 - `request list` has no
`--status`.)

### 2. Reconstruct one request - the full chain

`audit` accepts an AI request id, a task id, or a trace id and rebuilds identity -> policy evaluations -> prompt ->
response -> tool calls -> cost:

```bash
systemprompt infra logs audit <request-id> --full
systemprompt infra logs audit <request-id> --json     # machine-readable
```

For a quick single-request view scoped to one request id, `request show` is the lighter command:

```bash
systemprompt infra logs request show <request-id> --full
```

### 3. Follow the tool calls it spawned

If the request invoked MCP tools, follow them by trace:

```bash
systemprompt infra logs trace show <trace-id> --all          # steps, AI requests, MCP calls, artifacts
systemprompt infra logs trace list --status failed --limit 10
systemprompt infra logs trace list --has-mcp --tool <tool-name>
systemprompt infra logs tools list --limit 20                # raw MCP tool executions
```

### 4. Watch live while reproducing

```bash
systemprompt infra logs stream --since 30s                   # tail -f (alias: follow)
systemprompt infra logs view --level error --since 1h
```

### Typical workflow

1. `infra logs request list --since 1h` - find the request.
2. `infra logs audit <id> --full` - identity, governance decisions, prompt/response, tool calls, cost.
3. `infra logs trace show <trace-id> --all` - follow any tool calls it spawned.
4. `plugins mcp logs <server>` - if a tool failed, read the underlying MCP error.
