#!/bin/bash
# DEMO 1: HAPPY PATH — GOVERNANCE ALLOW + MCP TOOL CALL
# platform agent / developer_agent (admin scope, systemprompt MCP)
# Expected: Governance ALLOWS the tool call, then MCP returns real data
#
# This simulates a Claude Code PreToolUse hook workflow.
# Part 1 shows the governance check (ALLOW).
# Part 2 shows the tool execution that follows.
#
# What this does:
#   1. Loads auth token from demo/.token
#   2. POSTs to /api/public/hooks/govern simulating a PreToolUse hook
#      - agent_id=developer_agent (admin scope, allowed to use MCP tools)
#      - tool_name=mcp__systemprompt__systemprompt (clean tool input)
#      - Governance evaluates all rules → ALLOW
#   3. Calls the actual MCP tool via CLI to show what executes after ALLOW
#   4. Audits the governance_decisions table for the allow record
#
# Flow:
#   Claude Code hook fires → POST /hooks/govern → JWT auth → rules evaluate
#   → ALLOW → Claude Code proceeds → MCP tool executes → result returned
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

echo ""
echo "=========================================="
echo "  DEMO 1: HAPPY PATH"
echo "  governance ALLOW + MCP tool execution"
echo "=========================================="
echo ""

TOKEN="${1:-}"
if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument:"
  echo "  ./demo/01-happy-path.sh <TOKEN>"
  echo ""
  exit 1
fi

# ──────────────────────────────────────────────
#  PART 1: Governance check (PreToolUse hook)
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 1: Governance check (PreToolUse)"
echo ""
echo "  Simulating Claude Code PreToolUse hook:"
echo "    agent_id:   developer_agent"
echo "    tool_name:  mcp__systemprompt__systemprompt"
echo "    tool_input: {}"
echo ""
echo "  developer_agent has admin scope — this"
echo "  tool is allowed. Expecting: ALLOW"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__systemprompt",
    "agent_id": "developer_agent",
    "session_id": "demo-happy-path",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {}
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

# ──────────────────────────────────────────────
#  PART 2: MCP tool execution (what happens after ALLOW)
# ──────────────────────────────────────────────
echo ""
echo "------------------------------------------"
echo "  PART 2: MCP tool execution"
echo ""
echo "  Governance returned ALLOW, so Claude Code"
echo "  proceeds to execute the tool. Running:"
echo "    plugins mcp call systemprompt systemprompt \\"
echo "      -a '{\"command\":\"admin agents list\"}'"
echo "------------------------------------------"
echo ""

"$CLI" plugins mcp call systemprompt systemprompt -a '{"command":"admin agents list"}' 2>&1

# ──────────────────────────────────────────────
#  AUDIT: Verify governance decision
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions for this session"
echo "=========================================="
echo ""

echo "  Decision counts (session=demo-happy-path):"
"$CLI" infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = 'demo-happy-path' GROUP BY decision ORDER BY decision" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Detailed decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE session_id = 'demo-happy-path' ORDER BY created_at" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra db query \"SELECT * FROM governance_decisions WHERE session_id = 'demo-happy-path' ORDER BY created_at\""
echo ""
echo "  Part 1: ALLOWED (admin scope, clean input)"
echo "  Part 2: MCP tool returned real agent data"
echo ""
echo "  Now run: ./demo/02-refused-path.sh"
echo "=========================================="
