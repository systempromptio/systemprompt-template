#!/bin/bash
# DEMO 5: GOVERNANCE DENIED PATH
# Shows the backend rejecting a tool call for a user-scope agent
#
# What this does:
#   Part 1 — Direct API call:
#     1. Gets auth token from demo/.token (set by 00-preflight.sh)
#     2. POSTs directly to /api/public/hooks/govern with:
#        - tool_name: mcp__systemprompt__list_agents (admin-only MCP tool)
#        - agent_id: associate_agent (user scope)
#     3. Shows raw JSON response: permissionDecision: "deny"
#        - scope_check rule fails: user scope cannot access mcp__systemprompt__* tools
#
#   Part 2 — Live agent attempt:
#     1. Creates isolated context, messages associate_agent
#     2. Agent tries to call MCP tool → PreToolUse hook fires
#     3. Governance returns DENY → Claude Code blocks the tool call
#     4. Agent reports it cannot execute the operation
#
# Flow:
#   Agent → MCP tool call → PreToolUse hook → POST /hooks/govern
#   → JWT auth → scope=user → scope_check FAILS → DENY → tool blocked
#
# Contrast with Demo 04:
#   Demo 04: admin scope → rules pass → ALLOW
#   Demo 05: user scope → scope_check fails → DENY
#
# Cost: ~$0.01 (one AI call + one direct API call, no tool execution)

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
echo "  DEMO 5: GOVERNANCE — DENIED PATH"
echo "  associate_agent — user scope, governance blocks MCP"
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

# Get a token: argument, demo/.token (set by 00-preflight.sh), or fail
TOKEN="${1:-}"
TOKEN_FILE="$SCRIPT_DIR/.token"
if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo "ERROR: No token. Run demo/00-preflight.sh first, or pass token as argument." >&2
  exit 1
fi

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

# ──────────────────────────────────────────────
#  AUDIT: Verify governance denial
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions"
echo "=========================================="
echo ""

echo "  Most recent governance decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 3" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: decision=deny, policy=scope_restriction"
echo "  scope_check FAILED: user scope cannot access mcp__systemprompt__* tools"

echo ""
echo "------------------------------------------"
echo "  Trace:"
echo "------------------------------------------"
echo ""
TRACE_ID=$("$CLI" infra logs trace list --limit 1 2>&1 | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
if [[ -n "$TRACE_ID" ]]; then
  echo "  $TRACE_ID"
  "$CLI" infra logs trace show "$TRACE_ID" --all 2>&1 | head -20
fi

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra logs trace show $TRACE_ID --all"
echo "  infra db query \"SELECT * FROM governance_decisions ORDER BY created_at DESC LIMIT 5\""
echo ""
echo "  The governance endpoint DENIED the call."
echo "  user scope → scope_check failed → tool blocked."
echo ""
echo "  Now run: ./demo/06-governance-secret-breach.sh"
echo "=========================================="
