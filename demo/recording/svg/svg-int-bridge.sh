#!/bin/bash
# SVG RECORDING: Claude Desktop & Cowork
# Skills persist across sessions — governed, synced, delivered via MCP.
set -e
source "$(dirname "$0")/_colors.sh"

header "COWORK" "Skills synced to Claude Desktop — governed at every step"
pause 1

# ── Skill catalogue ──
subheader "Skill catalogue" "every skill the platform ships, synced to the database"
pause 0.3

type_cmd "systemprompt core skills list"
pause 0.3

"$CLI" core skills list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -16 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
pause 1.2

divider

# ── Sync status ──
subheader "Sync status" "disk vs database — are skills in sync?"
pause 0.3

type_cmd "systemprompt core skills status"
pause 0.3

"$CLI" core skills status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
check "skills synced between disk config and database"
pause 1.2

divider

# ── Inspect one skill ──
subheader "Inspect a skill" "full config + instruction body"
pause 0.3

type_cmd "systemprompt core skills show use_dangerous_secret"
pause 0.3

"$CLI" core skills show use_dangerous_secret --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -16 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1.2

divider

# ── MCP delivery ──
subheader "MCP delivery" "skills reach Claude Desktop through governed MCP servers"
pause 0.3

type_cmd "systemprompt plugins mcp status"
pause 0.3

"$CLI" plugins mcp status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
check "skills synced, governed, delivered to any MCP client"
pause 1.5
