#!/bin/bash
# DEMO 1: HAPPY PATH
# platform agent / developer_agent (admin scope, systemprompt MCP)
# Expected: Returns a real list of agents via MCP tool call + artifact
#
# What this does:
#   1. Creates an isolated execution context (ContextId) via `core contexts create`
#   2. Sends a message to developer_agent asking it to list agents
#      - developer_agent has admin scope → all MCP tools available
#      - The agent calls the systemprompt MCP server's list_agents tool
#   3. Waits synchronously (--blocking --timeout 60) for the AI response
#   4. Retrieves the structured artifact the agent produced
#      - Artifacts are typed data (JSON) that any surface can render
#
# Flow:
#   CLI → create context → message agent → AI processes → MCP tool call
#   → systemprompt CLI executes → result returned → artifact stored → displayed
#
# IDs created: ContextId, TraceId (per AI request), SessionId
# Cost: ~$0.01 (one AI call on Gemini Flash, multi-turn tool use)

set -e

# Resolve the CLI binary
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: cargo build" >&2
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 1: HAPPY PATH"
echo "  platform agent — admin scope, has MCP"
echo "=========================================="
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 1 - Happy Path $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')

if [[ -z "$CONTEXT_ID" ]]; then
  echo "WARNING: Could not create context, running without isolation"
  "$CLI" admin agents message developer_agent \
    -m "List all agents running on this platform" \
    --blocking --timeout 60
else
  echo "Context: $CONTEXT_ID"
  echo ""
  "$CLI" admin agents message developer_agent \
    -m "List all agents running on this platform" \
    --context-id "$CONTEXT_ID" \
    --blocking --timeout 60

  # Retrieve and display the artifact
  echo ""
  echo "=========================================="
  echo "  ARTIFACT — structured data from MCP tool"
  echo "=========================================="
  echo ""
  echo "The agent produced a typed artifact. This is"
  echo "structured data that can be rendered by any"
  echo "agent surface — web dashboard, mobile app,"
  echo "Slack bot, or CLI."
  echo ""

  ARTIFACT_ID=$("$CLI" core artifacts list --context-id "$CONTEXT_ID" 2>&1 | grep -oP '"id":\s*"\K[^"]+' | head -1)

  if [[ -n "$ARTIFACT_ID" ]]; then
    "$CLI" core artifacts show "$ARTIFACT_ID" --full
  else
    echo "(No artifact found — the agent may have returned inline text)"
  fi
fi

# ──────────────────────────────────────────────
#  AUDIT: Verify what just happened
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Verifying trace"
echo "=========================================="
echo ""

TRACE_ID=$("$CLI" infra logs trace list --limit 1 2>&1 | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)

if [[ -n "$TRACE_ID" ]]; then
  echo "  Trace: $TRACE_ID"
  echo ""
  "$CLI" infra logs trace show "$TRACE_ID" --all 2>&1 | head -40
  echo ""
  echo "  Expected: ~11 events, 3 AI requests, 1 MCP tool call"
else
  echo "  (No trace found)"
fi

echo ""
echo "------------------------------------------"
echo "  Cost breakdown:"
echo "------------------------------------------"
echo ""
"$CLI" analytics costs breakdown --by agent 2>&1 | head -10

echo ""
echo "------------------------------------------"
echo "  Governance log (most recent):"
echo "------------------------------------------"
echo ""
LOGFILE=$(ls -t /tmp/systemprompt-governance-*.log 2>/dev/null | head -1)
if [[ -n "$LOGFILE" ]]; then
  tail -5 "$LOGFILE"
else
  echo "  (No governance log found)"
fi

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra logs trace show $TRACE_ID --all"
echo "  analytics costs breakdown --by agent"
echo "  core artifacts list --context-id $CONTEXT_ID"
echo ""
echo "  Now run: ./demo/02-refused-path.sh"
echo "=========================================="
