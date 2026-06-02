# Trace Gateway Messages

Follow a single inference message through the AI gateway - from the client's `/v1/messages` call, through model routing and the governance pipeline, to the provider and back - to debug routing, denials, latency, or cost.

## When to Use

Use this skill when a Cowork session or any Anthropic-SDK client sends a message and you need to see exactly what the gateway did with it: which provider it routed to, whether governance allowed it, what the provider returned, and what it cost. Every `/v1/messages` hit lands one audited row carrying `user_id`, `tenant_id`, `session_id`, and `trace_id`, so a single id reconstructs the whole chain.

## How to Use

### The gateway path

Clients call `/v1/messages` on the gateway. The gateway routes by model pattern to a provider (see the `gateway.routes` block in the active profile - e.g. `claude-*` -> anthropic, `gpt-*` -> openai), runs the synchronous governance pipeline, forwards to the provider, and audits the result. `infra logs request` is that gateway message log.

### 1. List recent gateway messages

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --model claude
systemprompt infra logs request list --since 24h --provider anthropic
```

Each row is one `/v1/messages` hit: user, model, token counts, cost, latency, status.

### 2. Inspect one message

```bash
systemprompt infra logs request show <request-id> --full
```

### 3. Trace the full chain

`audit` accepts an AI request id, task id, or trace id and reconstructs identity -> policy evaluations -> prompt -> response -> cost:

```bash
systemprompt infra logs audit <id> --full
```

If the message spawned MCP tool calls, follow them by `trace_id`:

```bash
systemprompt infra logs trace show <trace-id>
systemprompt infra logs trace list --status failed --limit 10
```

### 4. Aggregate gateway traffic

```bash
systemprompt analytics requests stats           # volume, latency, error rate
systemprompt analytics requests models          # per-model breakdown (routing)
systemprompt analytics conversations list       # grouped by conversation
systemprompt analytics sessions live            # active sessions right now
systemprompt analytics costs summary            # spend rollup
```

### 5. Watch live while reproducing

```bash
systemprompt infra logs stream --since 30s
```

### Typical workflow

1. `infra logs request list --since 1h` - find the message.
2. `infra logs request show <id> --full` - see its gateway detail and routing.
3. `infra logs audit <id> --full` - get identity, governance decisions, prompt/response, and cost.
4. `infra logs trace show <trace-id>` - follow any tool calls the message spawned.
