#!/bin/bash
# MCP: SERVER MANAGEMENT — List servers, check status, view logs
# Shows MCP server configuration, runtime status, and server logs.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/mcp/01-mcp-servers.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "MCP: SERVER MANAGEMENT" "List servers, check status, view logs"

subheader "STEP 1: List MCP Servers"
run_cli_indented plugins mcp list

subheader "STEP 2: MCP Server Status"
run_cli_indented plugins mcp status

subheader "STEP 3: MCP Server Logs — systemprompt"
run_cli_head 20 plugins mcp logs systemprompt

subheader "STEP 4: MCP Server Logs — skill-manager"
run_cli_head 20 plugins mcp logs skill-manager

header "MCP SERVER DEMO COMPLETE"
