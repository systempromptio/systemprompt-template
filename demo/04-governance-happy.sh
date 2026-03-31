#!/bin/bash
# DEMO 4: GOVERNANCE HAPPY PATH
# developer_agent (admin scope, systemprompt MCP)
# Expected: Governance hook calls backend → backend evaluates rules → ALLOW → tool executes
#
# What this does:
#   1. Creates an isolated context
#   2. Messages developer_agent (admin scope) to list agents
#   3. When the agent calls an MCP tool, the PreToolUse hook fires:
#      a. Hook POSTs to /api/public/hooks/govern with tool_name + agent_id
#      b. Backend validates JWT, resolves agent scope (admin)
#      c. Rule engine evaluates: scope_check, secret_injection, rate_limit
#      d. All rules PASS → returns permissionDecision: "allow"
#      e. Hook allows Claude Code to proceed with tool execution
#   4. Shows the governance log file (/tmp/systemprompt-governance-*.log)
#
# Flow:
#   Agent → MCP tool call → PreToolUse hook → POST /hooks/govern
#   → JWT auth → scope=admin → rules pass → ALLOW → tool executes
#
# Key difference from Demo 01:
#   Demo 01 shows the agent result; Demo 04 shows the governance decision
#
# Cost: ~$0.01 (one AI call with governance overhead)

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
echo "  developer_agent — admin scope, has MCP"
echo ""
echo "  Flow:"
echo "    1. Agent calls MCP tool"
echo "    2. PreToolUse hook fires (synchronous)"
echo "    3. Hook POSTs to /api/public/hooks/govern"
echo "    4. Backend evaluates governance rules"
echo "    5. Backend returns HTTP 200: decision=allow"
echo "    6. Hook outputs permissionDecision=allow"
echo "    7. Claude Code proceeds with tool execution"
echo "=========================================="
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 4 - Governance Approved $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')

if [[ -z "$CONTEXT_ID" ]]; then
  echo "WARNING: Could not create context, running without isolation"
  "$CLI" admin agents message developer_agent \
    -m "List all agents running on this platform" \
    --blocking --timeout 60
else
  echo "Context: $CONTEXT_ID"
  echo ""
  "$CLI" admin agents message developer_agent \
    -m "List all agents running on this platform" \
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
#  AUDIT: Verify governance decision
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions"
echo "=========================================="
echo ""

echo "  Most recent governance decision:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason, evaluated_rules FROM governance_decisions ORDER BY created_at DESC LIMIT 1" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: decision=allow, all 3 rules passed"
echo "  (scope_check: PASS, secret_injection: PASS, rate_limit: PASS)"

echo ""
echo "------------------------------------------"
echo "  Trace:"
echo "------------------------------------------"
echo ""
TRACE_ID=$("$CLI" infra logs trace list --limit 1 2>&1 | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
if [[ -n "$TRACE_ID" ]]; then
  echo "  $TRACE_ID"
  "$CLI" infra logs trace show "$TRACE_ID" --all 2>&1 | head -30
fi

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra logs trace show $TRACE_ID --all"
echo "  infra db query \"SELECT * FROM governance_decisions ORDER BY created_at DESC LIMIT 3\""
echo ""
echo "  The governance endpoint APPROVED the call."
echo "  admin scope → all rules passed → tool executed."
echo ""
echo "  Now run: ./demo/05-governance-denied.sh"
echo "=========================================="
