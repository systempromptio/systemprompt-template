#!/bin/bash
# domain-mcp — MCP plumbing
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-mcp" "MCP servers as first-class tool providers"
pause 0.8

type_cmd "systemprompt plugins mcp list"
pause 0.2
"$CLI" plugins mcp list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt plugins mcp show systemprompt"
pause 0.2
"$CLI" plugins mcp show systemprompt --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -20 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "MCP as a library primitive, not a side process"
pause 1.2
