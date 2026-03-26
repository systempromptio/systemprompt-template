#!/bin/bash
# DEMO 5: GOVERNANCE DENIED PATH
# Shows the backend rejecting a tool call for a user-scope agent
#
# Part 1: Direct API call to show the raw governance response
# Part 2: associate_agent (user scope, has MCP) attempts tool call → blocked by governance

set -e

# Resolve the CLI binary
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: cargo build" >&2
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 5: GOVERNANCE — DENIED PATH"
echo "  associate_agent — user scope, has MCP"
echo ""
echo "  Flow:"
echo "    1. Agent calls MCP tool"
echo "    2. PreToolUse hook fires (synchronous)"
echo "    3. Hook POSTs to /api/public/hooks/govern"
echo "    4. Backend evaluates governance rules"
echo "    5. Backend returns HTTP 200: decision=deny"
echo "    6. Hook outputs permissionDecision=deny"
echo "    7. Claude Code BLOCKS the tool call"
echo "=========================================="
echo ""

# Part 1: Show raw governance response
echo "------------------------------------------"
echo "  PART 1: Raw governance API response"
echo "  (calling /api/public/hooks/govern directly)"
echo "------------------------------------------"
echo ""

# Get a token for the API call
TOKEN=$("$CLI" cloud auth token 2>/dev/null || echo "")

if [[ -n "$TOKEN" ]]; then
  curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "hook_event_name": "PreToolUse",
      "tool_name": "mcp__systemprompt__list_agents",
      "agent_id": "associate_agent",
      "session_id": "demo-governance-denied"
    }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
else
  echo "(Skipping direct API call — no auth token available)"
fi

echo ""
echo "------------------------------------------"
echo "  PART 2: Live agent attempt"
echo "  associate_agent tries to use admin tools"
echo "------------------------------------------"
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 5 - Governance Denied $(date +%H:%M:%S)" 2>&1)
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

# Show governance log
echo ""
echo "=========================================="
echo "  GOVERNANCE LOG"
echo "=========================================="
echo ""
LOGFILE=$(ls -t /tmp/systemprompt-governance-*.log 2>/dev/null | head -1)
if [[ -n "$LOGFILE" ]]; then
  tail -5 "$LOGFILE"
else
  echo "(No governance log found)"
fi

echo ""
echo "=========================================="
echo "  The governance endpoint DENIED the call."
echo "  user scope → scope_check failed → tool blocked."
echo "  The agent could not execute admin-only MCP tools."
echo ""
echo "  Now run: ./demo/03-audit-trail.sh"
echo "=========================================="
