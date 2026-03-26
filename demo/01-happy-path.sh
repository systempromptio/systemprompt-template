#!/bin/bash
# DEMO 1: HAPPY PATH
# platform agent / developer_agent (admin scope, systemprompt MCP)
# Expected: Returns a real list of agents via MCP tool call + artifact

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
echo "  DEMO 1: HAPPY PATH"
echo "  platform agent — admin scope, has MCP"
echo "=========================================="
echo ""

# Create a fresh isolated context
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo 1 - Happy Path $(date +%H:%M:%S)" 2>&1)
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

  # Retrieve and display the artifact
  echo ""
  echo "=========================================="
  echo "  ARTIFACT — structured data from MCP tool"
  echo "=========================================="
  echo ""
  echo "The agent produced a typed artifact. This is"
  echo "structured data that can be rendered by any"
  echo "agent surface — web dashboard, mobile app,"
  echo "Slack bot, or CLI."
  echo ""

  ARTIFACT_ID=$("$CLI" core artifacts list --context-id "$CONTEXT_ID" 2>&1 | grep -oP '"id":\s*"\K[^"]+' | head -1)

  if [[ -n "$ARTIFACT_ID" ]]; then
    "$CLI" core artifacts show "$ARTIFACT_ID" --full
  else
    echo "(No artifact found — the agent may have returned inline text)"
  fi
fi

echo ""
echo "=========================================="
echo "  Now run: ./demo/02-refused-path.sh"
echo "=========================================="
