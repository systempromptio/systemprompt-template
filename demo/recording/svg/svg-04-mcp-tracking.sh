#!/bin/bash
# SVG RECORDING: MCP Access Tracking
# OAuth-authenticated tool calls, tracked and audited.
set -e
source "$(dirname "$0")/_colors.sh"

header "MCP ACCESS TRACKING" "OAuth authentication + tool execution + audit"
pause 1

# ── MCP tool call 1 ──
subheader "skill-manager server" "OAuth → list_plugins"
pause 0.5

type_cmd "systemprompt plugins mcp call skill-manager list_plugins"
pause 0.5

OUTPUT=$("$CLI" plugins mcp call skill-manager list_plugins --args '{}' --profile "$PROFILE" 2>&1 | grep -v "^\[profile")

echo "$OUTPUT" | color_json
echo ""
pass "OAuth authenticated → tool executed"
pause 2

divider

# ── MCP tool call 2 ──
subheader "systemprompt server" "OAuth → list_agents"
pause 0.5

type_cmd "systemprompt plugins mcp call systemprompt list_agents"
pause 0.5

OUTPUT=$("$CLI" plugins mcp call systemprompt list_agents --args '{}' --profile "$PROFILE" 2>&1 | grep -v "^\[profile")

echo "$OUTPUT" | color_json
echo ""
pass "OAuth authenticated → tool executed"
pause 2

divider

# ── Activity log ──
subheader "Activity Log" "Database record of all MCP interactions"
pause 0.5

type_cmd "systemprompt infra db query \"SELECT action, entity_type, entity_name FROM user_activity ORDER BY created_at DESC LIMIT 6\""
pause 0.5

"$CLI" infra db query \
  "SELECT action, entity_type, entity_name FROM user_activity ORDER BY created_at DESC LIMIT 6" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | while IFS= read -r line; do
  echo -e "    ${CYAN}${line}${RESET}"
done

pause 2

divider

echo -e "  ${CYAN}${BOLD}Every MCP call: authenticated, executed, tracked.${RESET}"
echo ""
pause 2
