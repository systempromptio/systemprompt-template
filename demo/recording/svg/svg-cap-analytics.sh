#!/bin/bash
# SVG RECORDING: Analytics & Observability
# Every AI action, traced and costed. Merged from svg-03-audit + svg-05-tracing.
set -e
source "$(dirname "$0")/_colors.sh"

header "ANALYTICS & OBSERVABILITY" "Every AI action, traced and costed"
pause 1

# ── Audit trail ──
subheader "Audit Trail" "every governance decision queryable via SQL"
pause 0.3

type_cmd "systemprompt infra db query \"SELECT decision, tool_name, agent_id, policy FROM governance_decisions ORDER BY created_at DESC LIMIT 5\""
pause 0.5

"$CLI" infra db query \
  "SELECT decision, tool_name, agent_id, policy FROM governance_decisions ORDER BY created_at DESC LIMIT 5" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile\|^\[2m" | head -20 | while IFS= read -r line; do
  if echo "$line" | grep -q '"allow"'; then
    echo -e "    ${GREEN}${line}${R}"
  elif echo "$line" | grep -q '"deny"'; then
    echo -e "    ${RED}${line}${R}"
  else
    echo -e "    ${CYAN}${line}${R}"
  fi
done
pause 1.5

divider

# ── Execution traces ──
subheader "Execution Traces" "every event logged with typed IDs"
pause 0.3

type_cmd "systemprompt infra logs trace list --limit 3"
pause 0.3
"$CLI" infra logs trace list --limit 3 --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1.2

divider

# ── Cost attribution ──
subheader "Cost Attribution" "per-agent, per-model AI spend"
pause 0.3

type_cmd "systemprompt analytics costs breakdown --by agent"
pause 0.3
"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1.2

divider

# ── Overview ──
subheader "Dashboard Overview" "conversations, agents, requests, sessions — one query"
pause 0.3

type_cmd "systemprompt analytics overview"
pause 0.3
"$CLI" analytics overview --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""
check "every decision queryable, every token costed, every trace replayable"
pause 1.5
