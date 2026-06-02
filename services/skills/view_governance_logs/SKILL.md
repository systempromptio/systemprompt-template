# View Governance Logs

Read the governance spine - every inference call and every MCP tool call lands an audited row linking identity to agent to tool to result to cost.

## When to Use

Use this skill to investigate what an agent did, why a request failed, or how much something cost. The same CLI surface covers both AI requests (`/v1/messages`) and MCP tool calls - there is no separate "gateway logs" vs "tool logs".

## How to Use

### AI requests

Each `/v1/messages` hit is one row: user, model, token counts, cost, latency, status.

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 24h --provider anthropic
systemprompt infra logs request show <request-id>
systemprompt infra logs request stats
```

### Full audit for one request

Reconstruct the whole chain - identity, policy evaluations, prompt, response, and cost - from any AI request id, task id, or trace id:

```bash
systemprompt infra logs audit <id> --full
systemprompt infra logs audit <id> --json
```

### Tool-call traces

MCP tool calls flow PreToolUse -> decision -> spawn -> result. Filter failures here (the `--status` filter lives on traces, not on `request list`):

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace list --has-mcp --tool <tool-name>
systemprompt infra logs trace show <trace-id>
systemprompt infra logs tools list                  # list MCP tool executions
```

### Cost and usage rollups

Each analytics topic needs a verb (`summary`, `stats`, `list`, `trends`, `breakdown`/`models`/`show`):

```bash
systemprompt analytics overview
systemprompt analytics costs summary
systemprompt analytics costs breakdown
systemprompt analytics requests stats
systemprompt analytics agents list
systemprompt analytics tools stats
```

### Live tailing and errors

```bash
systemprompt infra logs stream --since 30s          # tail -f (alias: follow)
systemprompt infra logs view --level error --since 1h
```

### Typical workflow

1. `infra logs view --level error --since 1h` - find the error.
2. `infra logs trace list --status failed` - find the failed tool call.
3. `infra logs audit <id> --full` - get the full conversation and policy context.
4. `plugins mcp logs <server>` - get the underlying MCP tool error.
