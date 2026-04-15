---
name: "Extension: Hooks & Events"
description: "Hook catalog, lifecycle events, hook scripts, and event-driven automation"
---

# Extension: Hooks & Events

Hooks are event handlers that fire on system lifecycle events. They enable plugins to react to sessions, tool executions, agent activity, and user interactions.

---

## 1. Hook Events

| Event | Trigger | Use Cases |
|-------|---------|-----------|
| `tracking_session_start` | User session begins | Initialize tracking |
| `tracking_session_end` | User session ends | Finalize analytics |
| `tracking_post_tool_use` | Tool executed successfully | Log tool usage |
| `tracking_post_tool_use_failure` | Tool execution failed | Alert on failures |
| `tracking_subagent_start` | Sub-agent spawned | Track agent lifecycle |
| `tracking_subagent_stop` | Sub-agent stopped | Cleanup |
| `tracking_stop` | Application shutdown | Graceful cleanup |
| `tracking_user_prompt_submit` | User submits prompt | Input validation |

---

## 2. Hook Catalog

The hook catalog (`hook_catalog` table) stores hook definitions.

### Schema

```sql
CREATE TABLE IF NOT EXISTS hook_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    version TEXT NOT NULL DEFAULT '1.0.0',
    event TEXT NOT NULL,
    matcher TEXT,
    command TEXT NOT NULL,
    is_async BOOLEAN NOT NULL DEFAULT true,
    category TEXT,
    tags TEXT[],
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## 3. Hook Directory Structure

```
services/hooks/
  tracking_post_tool_use/
    my-hook.sh
  tracking_session_end/
    cleanup.sh
  tracking_session_start/
    init.sh
  tracking_stop/
    save-state.sh
  tracking_subagent_start/
    track-agent.sh
  tracking_user_prompt_submit/
    validate-input.sh
```

### Hook Script Format

```bash
#!/bin/bash
set -euo pipefail

# Environment variables available:
# $HOOK_EVENT - Event type
# $HOOK_DATA - JSON payload with event details
# $SYSTEMPROMPT_PROFILE - Profile path
```

---

## 4. Async vs Blocking Hooks

| Mode | Behavior | Use Case |
|------|----------|----------|
| Async (`is_async: true`) | Fire-and-forget | Logging, analytics, notifications |
| Blocking (`is_async: false`) | Waits for completion | Validation, authorization |

Blocking hooks must complete within 5 seconds.

---

## 5. Hook Debugging

```bash
systemprompt core hooks list

HOOK_EVENT=tracking_session_start HOOK_DATA='{"session_id":"test"}' bash services/hooks/tracking_session_start/my-hook.sh
```

---

## 6. Rules

| Rule | Rationale |
|------|-----------|
| Hook names must be unique | Catalog keyed by name |
| Scripts must be executable (`chmod +x`) | Shell execution requires permission |
| Use `set -euo pipefail` in scripts | Fail fast on errors |
| Prefer async hooks | Blocking hooks slow the pipeline |
| Hook data is JSON | Structured payload via `$HOOK_DATA` |
| Blocking hooks timeout at 5 seconds | Prevents hung pipelines |
