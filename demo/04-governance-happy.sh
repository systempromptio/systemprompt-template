#!/bin/bash
# DEMO 4: GOVERNANCE HAPPY PATH — DETAILED RULE EVALUATION
# Detailed rule evaluation — shows all 3 governance rules passing for an
# admin-scope agent with clean tool input.
#
# What this does:
#   1. Loads auth token from demo/.token (set by 00-preflight.sh)
#   2. POSTs directly to /api/public/hooks/govern with:
#      - agent_id: developer_agent (admin scope)
#      - tool_name: Read (clean, non-MCP tool)
#      - tool_input: {"file_path": "/src/main.rs"}
#   3. Shows full JSON response: permissionDecision: "allow"
#      - scope_check: admin scope passes for any tool
#      - secret_injection: tool_input contains no secrets
#      - rate_limit: well within limits
#   4. Audits governance_decisions table showing evaluated_rules column
#
# Key difference from Demo 01:
#   Demo 01 uses mcp__systemprompt__* tools via agent; Demo 04 uses a plain
#   "Read" tool via direct API call to exercise all 3 rules cleanly.
#
# Cost: Free (direct API call, no AI invocation)

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
echo "  DEMO 4: GOVERNANCE — HAPPY PATH"
echo "  Direct API call — all 3 rules pass"
echo ""
echo "  Flow:"
echo "    1. POST /api/public/hooks/govern"
echo "    2. agent_id=developer_agent (admin scope)"
echo "    3. tool_name=Read, tool_input={file_path}"
echo "    4. scope_check: PASS (admin scope)"
echo "    5. secret_injection: PASS (clean input)"
echo "    6. rate_limit: PASS (within limits)"
echo "    7. permissionDecision: allow"
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

echo "------------------------------------------"
echo "  Governance API response"
echo "  POST /api/public/hooks/govern"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "tool_input": {"file_path": "/src/main.rs"},
    "agent_id": "developer_agent",
    "session_id": "demo-governance-happy"
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

# ──────────────────────────────────────────────
#  AUDIT: Verify governance decision
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions"
echo "=========================================="
echo ""

echo "  Most recent governance decision (with evaluated_rules):"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason, evaluated_rules FROM governance_decisions ORDER BY created_at DESC LIMIT 1" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: decision=allow, all 3 rules passed"
echo "  (scope_check: PASS, secret_injection: PASS, rate_limit: PASS)"

echo ""
echo "=========================================="
echo "  Now run: ./demo/05-governance-denied.sh"
echo "=========================================="
