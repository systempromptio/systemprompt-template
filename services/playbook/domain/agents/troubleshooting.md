---
title: "Agent Troubleshooting"
description: "Diagnose and fix agent issues: startup failures, auth errors, task problems, tool failures."
keywords:
  - agents
  - troubleshooting
  - debug
  - errors
  - diagnose
category: domain
---

# Agent Troubleshooting

Diagnose and fix agent issues. Config: `services/agents/*.yaml`

> **Help**: `{ "command": "core playbooks show domain_agents-troubleshooting" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Diagnostic Checklist

{ "command": "infra services status" }
{ "command": "admin agents list" }
{ "command": "admin agents status" }
{ "command": "admin agents registry" }
{ "command": "infra logs --limit 50" }

---

## Issue: Agent Not Responding

Symptoms: 502 Bad Gateway, connection refused, timeout

Step 1: Check agent list

{ "command": "admin agents list" }

Step 2: Check process status

{ "command": "admin agents status" }

Step 3: Check logs

{ "command": "admin agents logs my-assistant --limit 100" }

Step 4: Check port conflicts

```bash
lsof -i :9001
```

Solutions:

Not in list:

{ "command": "admin agents validate" }
{ "command": "cloud sync local agents --direction to-db -y" }

Disabled (`enabled=false`): Edit `services/agents/<name>.yaml`:

```yaml
enabled: true
```

Port in use:

```bash
lsof -i :9001
kill <PID>
```

---

## Issue: OAuth Authentication Failures

Symptoms: 401 Unauthorized, 403 Forbidden, invalid token

Step 1: Check security config

{ "command": "admin agents show my-assistant" }

Step 2: Check session

{ "command": "admin session status" }

Step 3: Check OAuth config

{ "command": "admin config show --section oauth" }

Step 4: View auth logs

{ "command": "infra logs --context oauth --limit 50" }

Solutions:

Public agent (no auth):

```yaml
security:
  - oauth2: ["anonymous"]
```

Session expired:

{ "command": "admin session logout" }
{ "command": "admin session login" }

Scope mismatch: Agent requires `admin` but user only has `user` role:

{ "command": "admin users show <user_id>" }

---

## Issue: Skill Not Recognized

Symptoms: Skill not used, not shown in details, "skill not found"

Step 1: List skills

{ "command": "core skills list" }

Step 2: Check skill exists

{ "command": "core skills show <skill_id>" }

Step 3: Check agent skills

{ "command": "admin agents show my-assistant" }

Step 4: Verify sync

{ "command": "core skills sync --direction to-db --dry-run" }

Solutions:

Skill doesn't exist: Create `services/skills/<id>/config.yaml`:

```yaml
skill:
  id: my_skill
  name: "My Skill"
  description: "Does something useful"
  version: "1.0.0"
  enabled: true
  tags:
    - my-tag
  examples:
    - "Do something"
```

{ "command": "core skills sync --direction to-db -y" }

ID mismatch: Ensure agent `skills.id` matches skill `skill.id` exactly

Disabled: Set `enabled: true` in skill config

---

## Issue: Task Stuck

Symptoms: Task never completes, stays pending/submitted

Step 1: Get task details

{ "command": "admin agents task <task_id>" }

Step 2: Check agent health

{ "command": "admin agents registry" }

Step 3: Check AI provider

{ "command": "admin config show --section ai" }

Step 4: View logs

{ "command": "infra logs --context agent --limit 100" }

States: pending -> submitted -> working -> completed

Stuck at pending: Agent hasn't picked up task

{ "command": "admin agents status" }
{ "command": "infra services restart agents" }

Stuck at working: Waiting for AI or tool

{ "command": "infra logs --context ai --limit 50" }
{ "command": "infra logs --context mcp --limit 50" }
{ "command": "plugins mcp status" }

Rate limited: Switch provider in `services/ai/config.yaml`:

```yaml
ai:
  default_provider: gemini
```

---

## Issue: Tool Execution Failures

Symptoms: "couldn't execute tool", tool errors, MCP errors

Step 1: List tools

{ "command": "admin agents tools my-assistant" }

Step 2: Check MCP status

{ "command": "plugins mcp list" }
{ "command": "plugins mcp status" }

Step 3: View MCP logs

{ "command": "plugins mcp logs <server_name>" }

Step 4: Check execution logs

{ "command": "infra logs --context mcp --limit 50" }

Solutions:

MCP not running:

{ "command": "plugins mcp start <server_name>" }
{ "command": "plugins mcp status" }

Tool not listed:

{ "command": "plugins mcp refresh" }
{ "command": "admin agents tools my-assistant" }

Timeout: Increase in `services/ai/config.yaml`:

```yaml
mcp:
  execution_timeout_ms: 60000
  retry_attempts: 3
```

Auth error:

{ "command": "plugins mcp show <server_name>" }
{ "command": "cloud secrets set MCP_API_KEY \"new-key\"" }

---

## Issue: A2A Protocol Errors

Symptoms: Invalid protocol version, unknown transport, interop failures

Step 1: Check agent card

{ "command": "admin agents show my-assistant" }

Step 2: Check A2A endpoint

```bash
curl -X GET http://localhost:8080/api/v1/agents/my-assistant/.well-known/agent.json
```

Step 3: View A2A logs

{ "command": "infra logs --context a2a --limit 50" }

Solutions:

Protocol version:

```yaml
card:
  protocolVersion: "0.3.0"
```

Transport:

```yaml
card:
  preferredTransport: "JSONRPC"
```

Capabilities:

```yaml
capabilities:
  streaming: true
  pushNotifications: false
  stateTransitionHistory: false
```

---

## Issue: System Prompt Not Applied

Symptoms: Behavior doesn't match prompt, ignores instructions

Step 1: Check current prompt

{ "command": "admin agents show my-assistant" }

Step 2: Verify sync

{ "command": "cloud sync local agents --direction to-db --dry-run" }

Step 3: Validate YAML

{ "command": "admin agents validate" }

Solutions:

Not synced:

{ "command": "cloud sync local agents --direction to-db -y" }

YAML formatting: Use pipe for multiline:

```yaml
metadata:
  systemPrompt: |
    Your system prompt here.
    Use pipe (|) for multiline.
    Maintain consistent indentation.
```

---

## Log Levels

| Level | Meaning |
|-------|---------|
| `error` | Something failed |
| `warn` | Potential issue |
| `info` | Normal operation |
| `debug` | Detailed info |

{ "command": "infra logs --level error" }
{ "command": "infra logs --context agent --level error" }
{ "command": "infra logs --follow" }

---

## Quick Reference

| Problem | First Command |
|---------|---------------|
| Not responding | `admin agents status` |
| Auth failures | `admin session status` |
| Skill issues | `core skills list` |
| Task stuck | `admin agents task <id>` |
| Tool failures | `plugins mcp status` |
| A2A errors | `admin agents show <name>` |
| Any issue | `infra logs --limit 50` |

---

## Related

-> See [Agent Operations](agents-operations.md)
-> See [CLI Agents](../cli/agents.md)
-> See [MCP Troubleshooting](mcp-troubleshooting.md)
-> See [AI Troubleshooting](ai-troubleshooting.md)
