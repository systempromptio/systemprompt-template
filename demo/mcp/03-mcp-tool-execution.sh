#!/bin/bash
# MCP: TOOL EXECUTION — Tool listings, execution logs
# Shows tools exposed by each MCP server, recent tool-call logs, and
# usage analytics.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/mcp/03-mcp-tool-execution.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "MCP: TOOL EXECUTION" "Tool listings, execution logs"

subheader "STEP 1: Tools on systemprompt MCP server"
run_cli_head 30 plugins mcp tools -s systemprompt

subheader "STEP 2: Tools on skill-manager MCP server"
run_cli_head 30 plugins mcp tools -s skill-manager

subheader "STEP 3: MCP Tool Execution Log"
run_cli_head 20 infra logs tools list --limit 10

subheader "STEP 4: Tool Usage Analytics"
run_cli_indented analytics tools stats

subheader "STEP 5: Tool Trends"
run_cli_indented analytics tools trends --since 7d

header "MCP TOOL EXECUTION DEMO COMPLETE"
