#!/bin/bash
# DEMO 2: REFUSED PATH — GOVERNANCE DENIES ADMIN TOOL
# This shows what happens when a genuinely user-scope caller tries to call an
# admin-only tool. The governance API denies it, and this script asserts the
# real decision instead of narrating one.
#
# Identity model (the honest part):
#   Governance derives access scope from the CALLER'S LIVE DB ROLES, not from
#   the agent_id in the payload. So we send the request with the user-scope
#   plugin token from demo/.token.user (minted by 00-preflight.sh for
#   demo_user@demo.local, whose DB role is `user`). That token resolves to User
#   scope, so scope_check genuinely denies mcp__systemprompt__* tools.
#   (The admin demo/.token would be ALLOWED here — admins are exempt — which is
#   why this demo must use the user-scope token.)
#
# What this does:
#   1. POST /api/public/hooks/govern with the user-scope token, simulating a
#      PreToolUse hook for associate_agent calling mcp__systemprompt__list_agents
#   2. Captures the JSON response and asserts permissionDecision == deny
#      (fails loudly if the backend does not actually deny)
#   3. Prints commentary on defense-in-depth (mapping + rules)
#   4. Queries governance_decisions table for the deny record
#
# Flow:
#   curl → POST /hooks/govern → JWT auth → DB role=user → scope_check → DENY
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
echo "  user-scope token tries an admin-only tool"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  Load the user-scope auth token (real User scope, DB-role derived)
# ──────────────────────────────────────────────
load_user_token "${1:-}"

# ──────────────────────────────────────────────
#  PART 1: Governance denies admin tool for a user-scope caller
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  Simulating PreToolUse hook:"
echo "  identity=demo_user (user scope, token-derived from DB role)"
echo "  agent=associate_agent (user scope)"
echo "  tool=mcp__systemprompt__list_agents"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-refused-path",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {}
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
echo ""
assert_decision "$RESPONSE" "deny" "scope_check denies admin tool for user scope"

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
assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = 'demo-refused-path' AND decision = 'deny'")" \
  1 "deny decision landed in audit for demo-refused-path"
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
