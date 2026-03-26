#!/bin/bash
# DEMO 2: REFUSED PATH
# revenue agent / associate_agent (user scope, no MCP servers)
# Expected: Agent refuses — does not have tool access

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
echo "  DEMO 2: REFUSED PATH"
echo "  revenue agent — user scope, no MCP"
echo "=========================================="
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 2 - Refused Path $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')

if [[ -z "$CONTEXT_ID" ]]; then
  echo "WARNING: Could not create context, running without isolation"
  "$CLI" admin agents message associate_agent \
    -m "List all agents running on this platform using the CLI tools" \
    --blocking --timeout 60
else
  echo "Context: $CONTEXT_ID"
  echo ""
  "$CLI" admin agents message associate_agent \
    -m "List all agents running on this platform using the CLI tools" \
    --context-id "$CONTEXT_ID" \
    --blocking --timeout 60
fi

echo ""
echo "=========================================="
echo "  Now run: ./demo/03-audit-trail.sh"
echo "=========================================="
