---
title: "Terminal Demo: Agent Tracing — Full Pipeline"
description: "Run a live agent through the platform runtime. See AI reasoning, MCP tool calls, artifact creation, and full execution tracing with typed IDs."
author: "systemprompt.io"
slug: "demo-terminal-agent-tracing"
keywords: "demo, terminal, agents, tracing, artifacts, mcp, pipeline"
kind: "guide"
public: true
tags: ["demo", "terminal", "agents", "tracing", "mcp", "artifacts"]
published_at: "2026-03-31"
updated_at: "2026-03-31"
after_reading_this:
  - "Run a live agent with MCP tool access via the CLI"
  - "Retrieve structured artifacts from agent responses"
  - "Inspect full execution traces with AI requests, MCP calls, and timing"
  - "View agent traces on the dashboard at /admin/traces"
related_docs:
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "Governance Decisions Demo"
    url: "/documentation/demo-terminal-agents"
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
  - title: "Agents"
    url: "/documentation/agents"
---

## Overview

This demo runs a live AI agent through the platform runtime. It is the only demo that uses `admin agents message` to send a real prompt to an agent with MCP tool access. The agent reasons about the request, calls MCP tools to gather data, creates a structured artifact, and the platform traces every step.

**Cost:** ~$0.01 (one AI call with tool use).

**Prerequisites:** Complete [Setup & Authentication](/documentation/demo-terminal-setup) first.

---

## Step 1: Create a Context

Contexts isolate conversations. Each agent interaction gets its own context so artifacts, traces, and history stay separate.

```bash
CONTEXT_OUTPUT=$(systemprompt core contexts create --name "Agents - Agent Tracing" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '"id":\s*"\K[^"]+' | head -1)
echo "Context: $CONTEXT_ID"
```

---

## Step 2: Message the Agent

Send a message to `developer_agent` asking it to list all agents on the platform. This agent has admin scope and the systemprompt MCP server configured, so it can query the platform directly.

```bash
systemprompt admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --context-id "$CONTEXT_ID" \
  --blocking --timeout 60
```

### What Happens

1. The platform routes the message to `developer_agent`
2. The agent sees its available MCP tools and decides to call one
3. The MCP tool executes against the live platform and returns results
4. The agent formats the response and creates a structured artifact
5. Every step is traced with typed IDs and timing

The `--blocking` flag waits for the full response. The `--timeout 60` flag sets a 60-second deadline.

---

## Step 3: Retrieve the Artifact

Artifacts are typed data objects created by agents. They are retrievable by any surface --- CLI, API, dashboard, or another agent.

### List Artifacts

```bash
systemprompt core artifacts list --context-id "$CONTEXT_ID"
```

### Show Full Artifact

```bash
ARTIFACT_ID="<from list output>"
systemprompt core artifacts show "$ARTIFACT_ID" --full
```

The artifact contains the structured agent listing with metadata. It persists beyond the conversation and can be referenced later.

---

## Step 4: Execution Trace

The platform traces every event in the agent execution pipeline. Retrieve the trace to see the full sequence.

### List Recent Traces

```bash
systemprompt infra logs trace list --limit 1
```

### Show Full Trace

```bash
TRACE_ID="<from list output>"
systemprompt infra logs trace show "$TRACE_ID" --all
```

### What the Trace Contains

A typical agent message trace includes approximately 11 events:

- **3 AI requests** --- the multi-turn tool-use loop (see "Why 3 AI Requests?" below)
- **1 MCP tool call** --- the systemprompt tool invocation with input/output
- **Event timestamps** --- start/end timing for each step in the pipeline
- **Typed IDs** --- every entity (context, trace, artifact, request) has a typed newtype ID

---

## Step 5: Cost Breakdown

See how much each agent costs across all interactions:

```bash
systemprompt analytics costs breakdown --by agent
```

This shows token usage, request counts, and cost per agent. The demo should show ~$0.01 for `developer_agent`.

---

## Dashboard

The trace data is also visible on the admin dashboard.

- **Trace detail:** `/admin/traces?session_id=<session_id>` --- shows the event timeline, governance decisions, and all entities created during execution
- **Events view:** `/admin/events` --- shows the latest agent execution events across all sessions

The trace detail page renders the full event sequence with timing, making it easy to see where time was spent and which tools were called.

---

## Why 3 AI Requests?

Agent tool use is a multi-turn conversation between the AI model and the platform:

1. **Request 1:** The AI receives the user message, sees its available MCP tools, and decides to call the `list_agents` tool
2. **Request 2:** The MCP tool returns its result. The AI processes the tool output and determines how to respond
3. **Request 3:** The AI formats the final response with the agent listing and creates a structured artifact

This is normal multi-turn tool use. Each step is traced and costed separately in the platform. More complex tasks with multiple tool calls will generate additional AI requests.

---

## Next

Continue to the [Governance API demo](/documentation/demo-terminal-governance) to see how the hook-based governance pipeline works independently of agent messaging.
