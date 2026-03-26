#!/bin/bash
# PREFLIGHT CHECK
# Run this before the presentation to confirm everything is up

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
echo "  PREFLIGHT CHECK"
echo "=========================================="
echo ""

"$CLI" infra services status

echo ""
echo "You should see services running:"
echo "  - 3 agents (platform, revenue, admin)"
echo "  - 2 MCP servers (systemprompt, skill-manager)"
echo ""
echo "If anything is wrong:"
echo ""
echo "  $CLI infra services cleanup --yes"
echo "  $CLI infra services start --kill-port-process"
echo ""
