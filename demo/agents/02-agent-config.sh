#!/bin/bash
# AGENTS: CONFIGURATION — Validation, tools, status
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/02-agent-config.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Show an agent's validation result (box table) and assert validity against the
# structured --json card (sections[].heading=="valid"), failing loudly otherwise.
validate_agent() {
  local agent="$1"
  cmd "systemprompt admin agents validate $agent"
  "$CLI" admin agents validate "$agent" --profile "$PROFILE" 2>&1 | sed 's/^/  /'
  echo ""
  assert_eq "$(cli_json admin agents validate "$agent" \
    | jq -r '.sections[]|select(.heading=="valid").content')" \
    "true" "$agent configuration valid"
  echo ""
}

header "AGENTS: CONFIGURATION" "Validation and MCP tool inventory for both demo agents"

subheader "STEP 1: Agent Process Status"
run_cli_indented admin agents status

subheader "STEP 2: Validate developer_agent"
validate_agent developer_agent

subheader "STEP 3: MCP Tools Available to developer_agent"
run_cli_head 30 admin agents tools developer_agent

subheader "STEP 4: Validate associate_agent"
validate_agent associate_agent

subheader "STEP 5: MCP Tools Available to associate_agent"
run_cli_head 30 admin agents tools associate_agent

header "AGENT CONFIG DEMO COMPLETE"
