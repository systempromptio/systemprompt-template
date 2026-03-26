#!/bin/bash
# DEMO 4: GOVERNANCE HAPPY PATH
# developer_agent (admin scope, systemprompt MCP)
# Expected: Governance hook calls backend → backend evaluates rules → ALLOW → tool executes

set -e

# Resolve the CLI binary
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

echo ""
echo "=========================================="
echo "  The governance endpoint approved the call."
echo "  admin scope → all rules passed → tool executed."
echo ""
echo "  Now run: ./demo/05-governance-denied.sh"
echo "=========================================="
