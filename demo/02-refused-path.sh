#!/bin/bash
# DEMO 2: REFUSED PATH
# revenue agent / associate_agent (user scope, no MCP servers)
# Expected: Agent refuses — does not have tool access
#
# What this does:
#   1. Creates an isolated execution context (same as demo 01)
#   2. Sends the SAME message to associate_agent (not developer_agent)
#      - associate_agent has user scope → no MCP servers mapped
#      - The agent sees zero tools in its tool list
#   3. The AI responds: "I do not have access to that tool"
#      - No governance hook fires because there's no tool call to govern
#      - Access is denied at the mapping level, not the rule level
#
# Flow:
#   CLI → create context → message agent → AI sees no tools → refuses
#
# Contrast with Demo 01:
#   Demo 01: admin scope → MCP tools available → tool executes → result
#   Demo 02: user scope → no MCP tools mapped → AI refuses naturally
#
# Cost: ~$0.01 (one AI call, no tool use)

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
echo "  DEMO 2: REFUSED PATH"
echo "  revenue agent — user scope, no MCP"
echo "=========================================="
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 2 - Refused Path $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')

if [[ -z "$CONTEXT_ID" ]]; then
  echo "WARNING: Could not create context, running without isolation"
  "$CLI" admin agents message associate_agent \
    -m "List all agents running on this platform using the CLI tools" \
    --blocking --timeout 60
else
  echo "Context: $CONTEXT_ID"
  echo ""
  "$CLI" admin agents message associate_agent \
    -m "List all agents running on this platform using the CLI tools" \
    --context-id "$CONTEXT_ID" \
    --blocking --timeout 60
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
  "$CLI" infra logs trace show "$TRACE_ID" --all 2>&1 | head -20
  echo ""
  echo "  Expected: ~4 events, 1 AI request, 0 MCP calls, 0 governance decisions"
  echo "  Access denied at MAPPING level — no tools available, no hooks fired."
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
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra logs trace show $TRACE_ID --all"
echo "  analytics costs breakdown --by agent"
echo ""
echo "  Now run: ./demo/03-audit-trail.sh"
echo "=========================================="
