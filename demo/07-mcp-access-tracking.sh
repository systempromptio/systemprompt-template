#!/bin/bash
# DEMO 7: MCP ACCESS TRACKING & GOVERNANCE
# Shows successful + rejected tool calls surfaced on the admin dashboard.
#
# Usage:
#   ./demo/07-mcp-access-tracking.sh <TOKEN> [profile]
#
# TOKEN: The plugin token from the dashboard install widget (top-right of /admin/).
#        Click the key icon, reveal, and copy.

set -e

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

TOKEN="${1:-}"
PROFILE="${2:-local}"
DASHBOARD_URL="http://localhost:8080/admin/"

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Usage: ./demo/07-mcp-access-tracking.sh <TOKEN> [profile]"
  echo ""
  echo "  Get the TOKEN from the dashboard install widget:"
  echo "  1. Open $DASHBOARD_URL"
  echo "  2. Click the key icon (top-right)"
  echo "  3. Reveal and copy the plugin token"
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
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"demo-7","tool_input":{"file_path":"/src/main.rs"}}' \
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
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"demo-7","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket"}}' \
  | python3 -m json.tool 2>/dev/null || echo "(no response)"

echo ""
echo "  ✗ DENIED — AWS access key detected in tool input"
echo ""

# ──────────────────────────────────────────────
#  PART 3: Successful MCP tool call
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 3: MCP tool call — AUTHENTICATED"
echo "  Admin calls skill-manager list_plugins"
echo "------------------------------------------"
echo ""

"$CLI" plugins mcp call skill-manager list_plugins --profile "$PROFILE" 2>&1 \
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
