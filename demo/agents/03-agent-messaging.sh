#!/bin/bash
# DEMO 9: AGENT TRACING — Full Pipeline with Artifacts & MCP
# Platform agent runtime: messaging, AI reasoning, MCP tool calls, artifacts, tracing.
#
# This is the ONLY demo that calls `admin agents message`.
#
# What this does:
#   1. Creates a context for the demo session
#   2. Messages the developer_agent asking it to list all agents
#   3. The agent reasons, calls MCP tools, and produces an artifact
#   4. Retrieves the artifact created by the agent
#   5. Shows the full execution trace (events, AI requests, tool calls)
#   6. Shows cost breakdown by agent
#
# This demo shows the platform agent runtime — separate from the governance
# hook workflow. It demonstrates: agent messaging, AI reasoning, MCP tool
# calls, artifact creation, and full execution tracing.
#
# Usage:
#   ./demo/09-agent-tracing.sh [profile]
#
# Cost: ~$0.01 (one AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

PROFILE="${1:-local}"

echo ""
echo "=========================================="
echo "  DEMO 9: AGENT TRACING"
echo "  Full Pipeline with Artifacts & MCP"
echo ""
echo "  This demo shows the platform agent runtime — separate"
echo "  from the governance hook workflow. It demonstrates:"
echo "    - Agent messaging"
echo "    - AI reasoning"
echo "    - MCP tool calls"
echo "    - Artifact creation"
echo "    - Full execution tracing"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  STEP 1: Create a context
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 1: Create a context"
echo "------------------------------------------"
echo ""

CONTEXT_NAME="Demo 9 - Agent Tracing $(date +%H:%M:%S)"
echo "  \$ systemprompt core contexts create --name \"$CONTEXT_NAME\""
echo ""

CONTEXT_OUTPUT=$("$CLI" core contexts create --name "$CONTEXT_NAME" --profile "$PROFILE" 2>&1)
echo "  $CONTEXT_OUTPUT"
echo ""

CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '"id":\s*"\K[^"]+' | head -1 || true)
if [[ -z "$CONTEXT_ID" ]]; then
  CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)
fi

if [[ -z "$CONTEXT_ID" ]]; then
  echo "  ERROR: Could not extract context ID from output" >&2
  exit 1
fi

echo "  Context ID: $CONTEXT_ID"
echo ""

# ──────────────────────────────────────────────
#  STEP 2: Message the developer_agent
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 2: Message the developer_agent"
echo "  Asking it to list all agents on the platform"
echo "------------------------------------------"
echo ""

echo "  \$ systemprompt admin agents message developer_agent \\"
echo "      -m \"List all agents running on this platform\" \\"
echo "      --context-id \"$CONTEXT_ID\" --blocking --timeout 60"
echo ""

set +e
MESSAGE_OUTPUT=$("$CLI" admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --context-id "$CONTEXT_ID" \
  --blocking --timeout 60 \
  --profile "$PROFILE" 2>&1)
MESSAGE_RC=$?
set -e

echo "  Agent response (truncated):"
echo "$MESSAGE_OUTPUT" | head -40 | sed 's/^/  /'
LINES=$(echo "$MESSAGE_OUTPUT" | wc -l)
if [[ "$LINES" -gt 40 ]]; then
  echo "  ... ($((LINES - 40)) more lines)"
fi
echo ""

# Fail loudly if the agent conversation errored — the CLI sometimes exits 0
# even when the underlying provider call failed, so grep the output too.
if [[ "$MESSAGE_RC" -ne 0 ]] \
   || echo "$MESSAGE_OUTPUT" | grep -qiE "API key not valid|API_KEY_INVALID|Failed to send message|Gemini API error|Agent returned error|Internal error"; then
  echo "  ERROR: agent conversation failed." >&2
  if echo "$MESSAGE_OUTPUT" | grep -qiE "API key not valid|API_KEY_INVALID"; then
    echo "  Cause: Gemini API key is invalid or missing." >&2
    echo "  Fix:   set a valid key in .systemprompt/profiles/$PROFILE/secrets.json" >&2
    echo "         under the \"gemini\" field, then restart services (just start)." >&2
  fi
  exit 1
fi

# ──────────────────────────────────────────────
#  STEP 3: Retrieve artifact
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 3: Retrieve artifact"
echo "------------------------------------------"
echo ""

echo "  \$ systemprompt core artifacts list --context-id \"$CONTEXT_ID\""
echo ""

ARTIFACTS_OUTPUT=$("$CLI" core artifacts list --context-id "$CONTEXT_ID" --profile "$PROFILE" 2>&1)
echo "$ARTIFACTS_OUTPUT" | head -20 | sed 's/^/  /'
echo ""

ARTIFACT_ID=$(echo "$ARTIFACTS_OUTPUT" | grep -oP '"id":\s*"\K[^"]+' | head -1 || true)
if [[ -z "$ARTIFACT_ID" ]]; then
  ARTIFACT_ID=$(echo "$ARTIFACTS_OUTPUT" | grep -oP '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)
fi

if [[ -n "$ARTIFACT_ID" ]]; then
  echo "  Artifact ID: $ARTIFACT_ID"
  echo ""
  echo "  \$ systemprompt core artifacts show \"$ARTIFACT_ID\" --full"
  echo ""
  "$CLI" core artifacts show "$ARTIFACT_ID" --full --profile "$PROFILE" 2>&1 | head -50 | sed 's/^/  /'
  echo ""
else
  echo "  No artifact found for this context (agent may not have created one)."
  echo ""
fi

# ──────────────────────────────────────────────
#  STEP 4: Show trace
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 4: Execution trace"
echo "------------------------------------------"
echo ""

echo "  \$ systemprompt infra logs trace list --limit 3"
echo ""

TRACE_OUTPUT=$("$CLI" infra logs trace list --limit 3 --profile "$PROFILE" 2>&1)
echo "$TRACE_OUTPUT" | head -20 | sed 's/^/  /'
echo ""

TRACE_ID=$(echo "$TRACE_OUTPUT" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1 || true)
if [[ -z "$TRACE_ID" ]]; then
  TRACE_ID=$(echo "$MESSAGE_OUTPUT" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1 || true)
fi

if [[ -n "$TRACE_ID" ]]; then
  echo "  Trace ID: $TRACE_ID"
  echo ""
  echo "  \$ systemprompt infra logs trace show \"$TRACE_ID\" --all"
  echo ""
  "$CLI" infra logs trace show "$TRACE_ID" --all --profile "$PROFILE" 2>&1 | head -60 | sed 's/^/  /'
  echo ""
else
  echo "  Could not extract trace_id — check infra logs trace list manually."
  echo ""
fi

echo "  Expected: ~11 events, 3 AI requests, 1 MCP tool call, structured artifact"
echo ""

# ──────────────────────────────────────────────
#  STEP 5: Cost breakdown
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 5: Cost breakdown by agent"
echo "------------------------------------------"
echo ""

echo "  \$ systemprompt analytics costs breakdown --by agent"
echo ""
"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 | head -30 | sed 's/^/  /'
echo ""

# ──────────────────────────────────────────────
#  STEP 6: Dashboard URL
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 6: Dashboard"
echo "------------------------------------------"
echo ""

SESSION_ID=$(echo "$MESSAGE_OUTPUT" | grep -oP '"session_id":\s*"\K[^"]+' | head -1 || true)
if [[ -n "$SESSION_ID" ]]; then
  echo "  Dashboard: http://localhost:8080/admin/traces?session_id=$SESSION_ID"
else
  echo "  Dashboard: http://localhost:8080/admin/events"
  echo "  (Check /admin/events for the latest agent execution traces)"
fi
echo ""

# ──────────────────────────────────────────────
#  WHY 3 AI REQUESTS?
# ──────────────────────────────────────────────
echo "=========================================="
echo "  WHY 3 AI REQUESTS?"
echo ""
echo "  1. AI receives the message, sees MCP tools available,"
echo "     decides to call the systemprompt tool (list_agents)"
echo ""
echo "  2. MCP tool returns the result, AI processes the"
echo "     tool output and determines the response"
echo ""
echo "  3. AI formats the final response with the agent"
echo "     listing and creates a structured artifact"
echo ""
echo "  This is normal multi-turn tool use. Each step is"
echo "  traced and costed separately in the platform."
echo "=========================================="
echo ""

echo "=========================================="
echo "  DEMO 9 COMPLETE"
echo ""
echo "  What we showed:"
echo "    1. Created a context for the agent session"
echo "    2. Messaged developer_agent (admin agents message)"
echo "    3. Agent called MCP tools, produced an artifact"
echo "    4. Retrieved the structured artifact"
echo "    5. Full execution trace with events and AI requests"
echo "    6. Cost breakdown by agent"
echo ""
echo "  Cost: ~\$0.01 (one AI call with tool use)"
echo "=========================================="
