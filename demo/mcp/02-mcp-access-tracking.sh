#!/bin/bash
# DEMO 7: MCP ACCESS TRACKING & GOVERNANCE
# Shows successful + rejected tool calls surfaced on the admin dashboard.
#
# What this does:
#   Part 1 — Governance ALLOWED:
#     curl POST /hooks/govern with clean Read tool input
#     → All rules pass → permissionDecision: "allow"
#
#   Part 2 — Governance DENIED:
#     curl POST /hooks/govern with AWS key in Bash command
#     → secret_scan rule triggers → permissionDecision: "deny"
#
#   Part 3 — MCP tool call (authenticated):
#     `plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'`
#     → OAuth authentication → tool executes → result returned
#     → Access recorded in user_activity table
#
#   Part 4 — Audit trail (database):
#     Queries governance_decisions table: shows allow/deny + reasons
#     Queries user_activity table: shows MCP access events with category='mcp_access'
#
#   Part 5 — Dashboard pointer:
#     Points to http://localhost:8080/admin/ where all events appear in:
#     - Policy Violations section (governance denials)
#     - MCP Server Access section (auth events + tool calls)
#     - Governance Decisions section (allow/deny with reasons)
#
# Usage:
#   ./demo/mcp/02-mcp-access-tracking.sh <TOKEN> [profile]
#
# TOKEN: The plugin token from the dashboard install widget (top-right of /admin/).
#        Click the key icon, reveal, and copy.
#
# Cost: Free (governance API calls + direct MCP calls, no AI)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

TOKEN="${1:-}"
PROFILE="${2:-$PROFILE}"  # positional override; default comes from _common.sh
DASHBOARD_URL="${BASE_URL}/admin/"

if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument:"
  echo "  ./demo/mcp/02-mcp-access-tracking.sh <TOKEN> [profile]"
  echo ""
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 7: MCP ACCESS & GOVERNANCE"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  PART 1: Governance — ALLOWED (clean input)
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 1: Governance — ALLOWED"
echo "  Admin agent reads a source file"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"demo-7","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/src/main.rs"}}')
printf '%s\n' "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(no response)"

echo ""
assert_decision "$RESPONSE" "allow" "clean Read input — all rules pass"
echo ""

# ──────────────────────────────────────────────
#  PART 2: Governance — DENIED (secret in input)
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 2: Governance — DENIED"
echo "  Agent tries to curl with an AWS access key"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"demo-7","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket"}}')
printf '%s\n' "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(no response)"

echo ""
assert_decision "$RESPONSE" "deny" "AWS access key in tool input — secret_scan denies"
echo ""

# ──────────────────────────────────────────────
#  PART 3: Successful MCP tool call
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 3: MCP tool call — AUTHENTICATED"
echo "  Admin calls systemprompt core skills list"
echo "------------------------------------------"
echo ""

MCP_JSON=$("$CLI" --json plugins mcp call systemprompt systemprompt \
  --args '{"command":"core skills list"}' --profile "$PROFILE" 2>/dev/null)
printf '%s\n' "$MCP_JSON" | jq -r '.sections[] | "  \(.heading): \(.content)"' 2>/dev/null \
  | grep -E 'server|tool|success' || true
MCP_OK=$(printf '%s' "$MCP_JSON" | jq -r '.sections[] | select(.heading=="success") | .content' 2>/dev/null)

echo ""
assert_eq "$MCP_OK" "true" "OAuth authenticated → MCP tool executed"
echo ""

sleep 1

# ──────────────────────────────────────────────
#  PART 4: Audit trail
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 4: Audit trail (database)"
echo "------------------------------------------"
echo ""

echo "  Governance decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""
echo "  MCP access events:"
"$CLI" infra db query \
  "SELECT action, entity_name, description FROM user_activity WHERE category = 'mcp_access' ORDER BY created_at DESC LIMIT 5" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""
assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = 'demo-7'")" \
  2 "demo-7 governance decisions landed (allow + deny)"

# ──────────────────────────────────────────────
#  PART 5: Dashboard
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo ""
echo "  On the dashboard:"
echo ""
echo "  • Policy Violations — governance denials"
echo "  • MCP Server Access — auth events"
echo "  • Governance Decisions — allow/deny + reasons"
echo ""
echo "  ➜  $DASHBOARD_URL"
echo ""
echo "=========================================="
