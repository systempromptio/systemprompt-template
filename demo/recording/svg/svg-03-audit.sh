#!/bin/bash
# SVG RECORDING: Audit Trail
# Every governance decision queryable in the database.
set -e
source "$(dirname "$0")/_colors.sh"

header "AUDIT TRAIL" "Every governance decision queryable via CLI"
pause 1

# ── Query governance decisions ──
type_cmd "systemprompt infra db query \"SELECT decision, tool_name, agent_id, policy FROM governance_decisions ORDER BY created_at DESC LIMIT 8\""
pause 0.5

OUTPUT=$("$CLI" infra db query \
  "SELECT decision, tool_name, agent_id, agent_scope, policy FROM governance_decisions ORDER BY created_at DESC LIMIT 8" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile")

# Colorize ALLOW/DENY in the output
echo "$OUTPUT" | while IFS= read -r line; do
  if echo "$line" | grep -q '"allow"'; then
    echo -e "    ${GREEN}${line}${RESET}"
  elif echo "$line" | grep -q '"deny"'; then
    echo -e "    ${RED}${line}${RESET}"
  else
    echo "    $line"
  fi
done

pause 2.5

divider

# ── Cost breakdown ──
subheader "Cost Attribution" "AI spend per agent"
pause 0.5

type_cmd "systemprompt analytics costs breakdown --by agent"
pause 0.5

"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | while IFS= read -r line; do
  echo -e "    ${CYAN}${line}${RESET}"
done

pause 2
