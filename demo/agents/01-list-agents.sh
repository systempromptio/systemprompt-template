#!/bin/bash
# AGENTS: DISCOVERY — List and inspect configured agents
# Shows admin vs core agent views and individual agent details.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/01-list-agents.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: DISCOVERY" "List and inspect configured agents"

subheader "STEP 1: List All Agents (Admin View)"
run_cli_indented admin agents list

subheader "STEP 2: List All Agents (Core View)"
run_cli_indented core agents list

subheader "STEP 3: Show Developer Agent"
run_cli_head 30 admin agents show developer_agent

subheader "STEP 4: Show Associate Agent"
run_cli_head 30 admin agents show associate_agent

info "Developer agent has admin scope — full MCP tool access."
info "Associate agent has user scope — restricted tools."

header "AGENT DISCOVERY DEMO COMPLETE"
