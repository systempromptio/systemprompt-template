#!/bin/bash
# AGENTS: DISCOVERY — List and inspect configured agents
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/01-list-agents.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: DISCOVERY" "List and inspect configured agents"

subheader "STEP 1: List All Agents"
LIST_OUTPUT=$("$CLI" admin agents list --profile "$PROFILE" 2>&1)
echo "$LIST_OUTPUT" | sed 's/^/  /'
echo ""

# Extract first agent id if any
FIRST_AGENT=$(echo "$LIST_OUTPUT" | grep -oP '"(name|id)":\s*"\K[^"]+' | head -1 || true)

subheader "STEP 2: Agent Process Status"
run_cli_indented admin agents status

if [[ -n "$FIRST_AGENT" ]]; then
  subheader "STEP 3: Show Agent: $FIRST_AGENT"
  run_cli_head 30 admin agents show "$FIRST_AGENT"
else
  subheader "STEP 3: Show Agent"
  info "No agents configured in services/agents/ — this template ships with"
  info "an empty agent registry. Agents are defined as YAML files under"
  info "services/agents/<id>.yaml and aggregated by services/config/config.yaml."
fi

header "AGENT DISCOVERY DEMO COMPLETE"
