#!/bin/bash
# MCP: SERVER MANAGEMENT — List servers, check status, view tools
# Shows MCP server runtime status and available tools.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/mcp/01-mcp-servers.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "MCP: SERVER MANAGEMENT" "List servers, check status, view tools"

subheader "STEP 1: MCP Server Status"
run_cli_indented plugins mcp status

subheader "STEP 2: MCP Tools — All Servers"
run_cli_head 30 plugins mcp tools

subheader "STEP 3: MCP Tools — systemprompt server only"
run_cli_head 30 plugins mcp tools --server systemprompt

subheader "STEP 4: MCP Tools — skill-manager server only"
run_cli_head 30 plugins mcp tools --server skill-manager

header "MCP SERVER DEMO COMPLETE"
