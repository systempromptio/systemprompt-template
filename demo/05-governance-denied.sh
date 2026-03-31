#!/bin/bash
# DEMO 5: GOVERNANCE DENIED PATH
# Shows the backend rejecting tool calls for a user-scope agent
#
# What this does:
#   Part 1 — Scope restriction denial:
#     1. Gets auth token from demo/.token (set by 00-preflight.sh)
#     2. POSTs directly to /api/public/hooks/govern with:
#        - tool_name: mcp__systemprompt__list_agents (admin-only MCP tool)
#        - agent_id: associate_agent (user scope)
#     3. Shows raw JSON response: permissionDecision: "deny"
#        - scope_check rule fails: user scope cannot access mcp__systemprompt__* tools
#
#   Part 2 — Blocklist denial:
#     1. POSTs directly to /api/public/hooks/govern with:
#        - tool_name: mcp__systemprompt__delete_agent (destructive MCP tool)
#        - agent_id: associate_agent (user scope)
#        - tool_input: {"agent_id":"test"}
#     2. Shows raw JSON response: permissionDecision: "deny"
#        - Both scope_check AND blocklist rules trigger on this call
#        - Blocklist catches destructive operations (delete_*) regardless of scope
#
# What Claude Code does with a deny response:
#   1. The PreToolUse hook returns permissionDecision: "deny" with a reason
#   2. Claude Code prints: [GOVERNANCE] <reason> — visible in the terminal
#   3. Claude Code BLOCKS the tool call — it never executes
#   4. The agent receives the denial reason and must explain it to the user
#   5. The denial is logged to governance_decisions for audit
#
# Flow:
#   Agent → MCP tool call → PreToolUse hook → POST /hooks/govern
#   → JWT auth → scope=user → scope_check FAILS → DENY → tool blocked
#
# Cost: Free (two direct API calls, no AI usage)

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

# Load token from demo/.token (set by 00-preflight.sh)
TOKEN="${1:-}"
TOKEN_FILE="$SCRIPT_DIR/.token"
if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo "ERROR: No token. Run demo/00-preflight.sh first, or pass token as argument." >&2
  exit 1
fi

# ──────────────────────────────────────────────
#  PART 1: Scope restriction — user cannot access admin tools
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 1: Scope restriction denial"
echo "  tool: mcp__systemprompt__list_agents"
echo "  agent: associate_agent (user scope)"
echo "  rule: scope_check — user scope cannot access mcp__systemprompt__* tools"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-governance-denied"
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "  ^ scope_check DENIED: user scope cannot call mcp__systemprompt__* tools"
echo ""

# ──────────────────────────────────────────────
#  PART 2: Blocklist — destructive tool blocked
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 2: Blocklist denial"
echo "  tool: mcp__systemprompt__delete_agent"
echo "  agent: associate_agent (user scope)"
echo "  rule: blocklist — destructive operations (delete_*) are always blocked"
echo "  Also triggers: scope_check (user scope + mcp__systemprompt__* tool)"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__delete_agent",
    "tool_input": {"agent_id": "test"},
    "agent_id": "associate_agent",
    "session_id": "demo-governance-denied-blocklist"
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "  ^ Blocked by BOTH scope_check and blocklist rules."
echo "  In Claude Code, the agent would see:"
echo "    [GOVERNANCE] Tool blocked: <reason>"
echo "    The tool call never executes. The agent must explain the denial."
echo ""

# ──────────────────────────────────────────────
#  GOVERNANCE LOG
# ──────────────────────────────────────────────
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
#  AUDIT: Verify governance denials
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions"
echo "=========================================="
echo ""

echo "  Most recent governance decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: two deny records"
echo "    1. scope_check: user scope cannot access mcp__systemprompt__list_agents"
echo "    2. blocklist/scope_check: destructive tool mcp__systemprompt__delete_agent blocked"

echo ""
echo "=========================================="
echo "  Now run: ./demo/06-governance-secret-breach.sh"
echo "=========================================="
