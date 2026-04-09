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

subheader "STEP 2: Validate MCP Servers"
run_cli_indented plugins mcp validate

subheader "STEP 3: MCP Tools — systemprompt server"
cmd "systemprompt plugins mcp tools systemprompt"
"$CLI" plugins mcp tools systemprompt --profile "$PROFILE" 2>&1 | head -30 | sed 's/^/  /' || info "Could not list tools."
echo ""

subheader "STEP 4: MCP Tools — skill-manager server"
cmd "systemprompt plugins mcp tools skill-manager"
"$CLI" plugins mcp tools skill-manager --profile "$PROFILE" 2>&1 | head -30 | sed 's/^/  /' || info "Could not list tools."
echo ""

header "MCP SERVER DEMO COMPLETE"
