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
#     → secret_injection rule triggers → permissionDecision: "deny"
#
#   Part 3 — MCP tool call (authenticated):
#     `plugins mcp call skill-manager list_plugins`
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
PROFILE="${2:-local}"
DASHBOARD_URL="http://localhost:8080/admin/"

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

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"demo-7","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/src/main.rs"}}' \
  | python3 -m json.tool 2>/dev/null || echo "(no response)"

echo ""
echo "  ✓ ALLOWED — clean input, all rules passed"
echo ""

# ──────────────────────────────────────────────
#  PART 2: Governance — DENIED (secret in input)
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 2: Governance — DENIED"
echo "  Agent tries to curl with an AWS access key"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"demo-7","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket"}}' \
  | python3 -m json.tool 2>/dev/null || echo "(no response)"

echo ""
echo "  ✗ DENIED — AWS access key detected in tool input"
echo ""

# ──────────────────────────────────────────────
#  PART 3: Successful MCP tool call
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 3: MCP tool call — AUTHENTICATED"
echo "  Admin calls systemprompt core skills list"
echo "------------------------------------------"
echo ""

"$CLI" plugins mcp call systemprompt systemprompt \
  --args '{"command":"core skills list"}' --profile "$PROFILE" 2>&1 \
  | grep -E '"success"|"server"|"tool"|"execution_time_ms"'

echo ""
echo "  ✓ OAuth authenticated → tool executed"
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
