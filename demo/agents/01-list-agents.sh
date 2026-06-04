#!/bin/bash
# AGENTS: DISCOVERY — List and inspect configured agents
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/01-list-agents.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: DISCOVERY" "List and inspect developer_agent and associate_agent side by side"

subheader "STEP 1: List All Agents"
run_cli_indented admin agents list
# Validate against the structured registry, not the rendered table.
assert_min "$(cli_json admin agents list | jq '.items | length')" \
  2 "agents registered (developer_agent + associate_agent)"

subheader "STEP 2: Agent Process Status"
run_cli_indented admin agents status

subheader "STEP 3: Show developer_agent (admin scope)"
run_cli_head 30 admin agents show developer_agent

subheader "STEP 4: Show associate_agent (user scope)"
run_cli_head 30 admin agents show associate_agent

header "AGENT DISCOVERY DEMO COMPLETE"
