#!/bin/bash
# Backward-compatible wrapper — delegates to mcp category
exec "$(dirname "$0")/mcp/02-mcp-access-tracking.sh" "$@"
