#!/bin/bash
# AGENTS: CONFIGURATION — Validation, tools, status
# Validates agent configs, lists available MCP tools, and checks process status.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/02-agent-config.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: CONFIGURATION" "Validation, tools, status"

subheader "STEP 1: Validate Agent Configs"
run_cli_indented admin agents validate developer_agent
run_cli_indented admin agents validate associate_agent

subheader "STEP 2: MCP Tools Available to Developer Agent"
run_cli_head 30 admin agents tools developer_agent

subheader "STEP 3: MCP Tools Available to Associate Agent"
run_cli_head 30 admin agents tools associate_agent

subheader "STEP 4: Agent Process Status"
run_cli_indented admin agents status

header "AGENT CONFIG DEMO COMPLETE"
