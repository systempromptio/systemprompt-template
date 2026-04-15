#!/bin/bash
# MCP: TOOL EXECUTION — Tool listings, execution logs
# Shows available tools per agent, execution logs, and usage analytics.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/mcp/03-mcp-tool-execution.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "MCP: TOOL EXECUTION" "Tool listings, execution logs"

subheader "STEP 1: Tools Available to Developer Agent"
run_cli_head 30 admin agents tools developer_agent

subheader "STEP 2: MCP Tool Execution Log"
run_cli_head 20 infra logs tools list --limit 10

subheader "STEP 3: Tool Usage Analytics"
run_cli_indented analytics tools stats

subheader "STEP 4: Tool Trends"
run_cli_indented analytics tools trends --since 7d

header "MCP TOOL EXECUTION DEMO COMPLETE"
