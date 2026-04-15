#!/bin/bash
# AGENTS: CONFIGURATION — Validation, tools, status
# Lists configured agents and shows process status. When agents are defined
# in services/agents/, also validates them and enumerates their MCP tools.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/02-agent-config.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: CONFIGURATION" "Validation, tools, status"

subheader "STEP 1: Agent Process Status"
run_cli_indented admin agents status

LIST_OUTPUT=$("$CLI" admin agents list --profile "$PROFILE" 2>&1)
FIRST_AGENT=$(echo "$LIST_OUTPUT" | grep -oP '"(name|id)":\s*"\K[^"]+' | head -1 || true)

if [[ -n "$FIRST_AGENT" ]]; then
  subheader "STEP 2: Validate Agent Config: $FIRST_AGENT"
  run_cli_indented admin agents validate "$FIRST_AGENT"

  subheader "STEP 3: MCP Tools Available to $FIRST_AGENT"
  run_cli_head 30 admin agents tools "$FIRST_AGENT"
else
  subheader "STEP 2: Agent Validation"
  info "No agents configured in services/agents/ — validation and tool"
  info "enumeration require at least one agent YAML."
  echo ""

  subheader "STEP 3: MCP Tools Available (across all servers)"
  run_cli_head 30 plugins mcp tools
fi

header "AGENT CONFIG DEMO COMPLETE"
