#!/bin/bash
# DEMO 2: REFUSED PATH — GOVERNANCE DENIES ADMIN TOOL
# This simulates what happens when a user-scope agent tries to call an
# admin-only tool. The governance API denies it.
#
# What this does:
#   1. Calls POST /api/public/hooks/govern simulating a Claude Code
#      PreToolUse hook for associate_agent calling mcp__systemprompt__list_agents
#   2. Governance evaluates scope_restriction rule:
#      associate_agent has user scope → admin tool is denied
#   3. Prints commentary on defense-in-depth (mapping + rules)
#   4. Queries governance_decisions table for the deny record
#
# Flow:
#   curl → POST /hooks/govern → JWT auth → scope_restriction check → DENY
#
# This is the second layer of defense. In a real Claude Code deployment,
# user-scope agents would never even see admin tools (mapping level).
# Governance provides backup enforcement at the rule level.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

echo ""
echo "=========================================="
echo "  DEMO 2: REFUSED PATH"
echo "  associate_agent tries an admin-only tool"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  Load auth token
# ──────────────────────────────────────────────
TOKEN="${1:-}"
if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument:"
  echo "  ./demo/governance/02-refused-path.sh <TOKEN>"
  echo ""
  exit 1
fi

# ──────────────────────────────────────────────
#  PART 1: Governance denies admin tool for user-scope agent
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  Simulating PreToolUse hook:"
echo "  agent=associate_agent (user scope)"
echo "  tool=mcp__systemprompt__list_agents"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-refused-path",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {}
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

# ──────────────────────────────────────────────
#  PART 2: Defense-in-depth commentary
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  DEFENSE-IN-DEPTH"
echo "=========================================="
echo ""
echo "  Two independent layers prevent unauthorized access:"
echo ""
echo "  Layer 1 — MAPPING (preventive)"
echo "    In a real Claude Code deployment, user-scope agents"
echo "    never even see admin tools. The MCP server mapping"
echo "    excludes them entirely — the tool does not appear"
echo "    in the agent's tool list."
echo ""
echo "  Layer 2 — GOVERNANCE RULES (detective + enforcement)"
echo "    Even if mapping were misconfigured, the scope_restriction"
echo "    rule evaluates every PreToolUse hook call. A user-scope"
echo "    agent calling an admin tool is denied and logged."
echo ""
echo "  Result: Two independent layers. Mapping prevents exposure."
echo "  Governance enforces policy. Neither depends on the other."
echo ""

# ──────────────────────────────────────────────
#  AUDIT: Query governance_decisions for the deny record
# ──────────────────────────────────────────────
echo "=========================================="
echo "  AUDIT: Governance decisions for this session"
echo "=========================================="
echo ""

echo "  Decision counts (session=demo-refused-path):"
"$CLI" infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = 'demo-refused-path' GROUP BY decision ORDER BY decision" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: 1 deny (scope_restriction)"
echo ""

echo "  Detailed decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE session_id = 'demo-refused-path' ORDER BY created_at" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  $CLI infra db query \"SELECT * FROM governance_decisions WHERE session_id = 'demo-refused-path' ORDER BY created_at\""
echo ""
echo "  associate_agent (user scope) was DENIED access to"
echo "  mcp__systemprompt__list_agents by scope_restriction rule."
echo ""
echo "  Now run: ./demo/governance/03-audit-trail.sh"
echo "=========================================="
