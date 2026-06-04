#!/bin/bash
# DEMO 5: GOVERNANCE DENIED PATH
# Shows the backend rejecting tool calls for a genuinely user-scope caller, and
# asserts the real decision rather than narrating one.
#
# Identity model (the honest part):
#   Governance derives access scope from the CALLER'S LIVE DB ROLES. This script
#   sends every deny request with the user-scope plugin token from
#   demo/.token.user (minted by 00-preflight.sh for demo_user@demo.local, DB role
#   `user`). That token resolves to User scope, so scope_check and tool_blocklist
#   genuinely deny. The admin demo/.token would be ALLOWED by both policies
#   (admins are exempt) — which is exactly why the deny demo uses this token.
#
# What this does:
#   Part 1 — Scope restriction denial:
#     1. Loads the user-scope token from demo/.token.user (set by 00-preflight.sh)
#     2. POSTs directly to /api/public/hooks/govern with:
#        - tool_name: mcp__systemprompt__list_agents (admin-only MCP tool)
#        - agent_id: associate_agent (user scope)
#     3. Captures the JSON response and asserts permissionDecision == deny
#        - scope_check rule fails: user scope cannot access mcp__systemprompt__* tools
#
#   Part 2 — Blocklist denial:
#     1. POSTs directly to /api/public/hooks/govern with:
#        - tool_name: delete_records (destructive name, NOT admin-prefixed)
#        - agent_id: associate_agent (user scope)
#        - tool_input: {"table":"users"}
#     2. Captures the JSON response and asserts permissionDecision == deny
#        - tool_blocklist is the policy that fires and is audited here. We use a
#          NON-admin-prefixed destructive name on purpose: an
#          mcp__systemprompt__delete_* tool would be short-circuited by
#          scope_check (it runs first), so the deny would be attributed to
#          scope_check, not tool_blocklist. delete_records passes scope_check
#          (not admin-only) and is then denied by tool_blocklist.
#        - tool_blocklist catches destructive names (delete/drop/destroy) for
#          user/non-admin scope (admins are exempt).
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

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

echo ""
echo "=========================================="
echo "  DEMO 5: GOVERNANCE — DENIED PATH"
echo "  demo_user — real user scope, governance blocks MCP"
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

# Load the user-scope token from demo/.token.user (set by 00-preflight.sh).
# Governance derives scope from the caller's live DB role, so this token
# resolves to User scope and the deny policies genuinely fire.
load_user_token "${1:-}"

# ──────────────────────────────────────────────
#  PART 1: Scope restriction — user cannot access admin tools
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 1: Scope restriction denial"
echo "  identity: demo_user (user scope, token-derived from DB role)"
echo "  tool: mcp__systemprompt__list_agents"
echo "  agent: associate_agent (user scope)"
echo "  rule: scope_check — user scope cannot access mcp__systemprompt__* tools"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-governance-denied",
    "cwd": "/var/www/html/systemprompt-template"
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
assert_decision "$RESPONSE" "deny" "scope_check denies admin tool for user scope"
echo ""

# ──────────────────────────────────────────────
#  PART 2: Blocklist — destructive tool blocked
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 2: Blocklist denial"
echo "  identity: demo_user (user scope, token-derived from DB role)"
echo "  tool: delete_records (destructive name, NOT admin-prefixed)"
echo "  agent: associate_agent (user scope)"
echo "  rule: tool_blocklist — destructive names (delete/drop/destroy) denied"
echo "        for user/non-admin scope (admins are exempt)"
echo "  Why not mcp__systemprompt__delete_*? scope_check runs first and would"
echo "  short-circuit it, attributing the deny to scope_check. A non-prefixed"
echo "  name passes scope_check and is denied by tool_blocklist — so the audit"
echo "  row genuinely reads policy=tool_blocklist."
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "delete_records",
    "tool_input": {"table": "users"},
    "agent_id": "associate_agent",
    "session_id": "demo-governance-denied-blocklist",
    "cwd": "/var/www/html/systemprompt-template"
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
assert_decision "$RESPONSE" "deny" "tool_blocklist denies destructive tool for user scope"
echo ""
echo "  ^ Blocked for user scope by tool_blocklist (policy=tool_blocklist in the audit)."
echo "  In Claude Code, the agent would see:"
echo "    [GOVERNANCE] Tool blocked: <reason>"
echo "    The tool call never executes. The agent must explain the denial."
echo ""

# ──────────────────────────────────────────────
#  GOVERNANCE LOG
# ──────────────────────────────────────────────
echo "=========================================="
echo "  GOVERNANCE LOG — recent deny decisions"
echo "=========================================="
echo ""
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE decision = 'deny' ORDER BY created_at DESC LIMIT 10" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

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
echo "    1. scope_check:    user scope cannot access mcp__systemprompt__list_agents"
echo "    2. tool_blocklist: destructive tool delete_records blocked for user scope"
echo ""
assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = 'demo-governance-denied' AND decision = 'deny'")" \
  1 "scope deny landed in audit (demo-governance-denied)"
assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = 'demo-governance-denied-blocklist' AND decision = 'deny'")" \
  1 "blocklist deny landed in audit (demo-governance-denied-blocklist)"

echo ""
echo "=========================================="
echo "  Now run: ./demo/governance/06-secret-breach.sh"
echo "=========================================="
