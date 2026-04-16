#!/bin/bash
# SVG RECORDING: Compliance
# Built to survive an audit. Identity-bound. Replayable. Retention-managed.
set -e
source "$(dirname "$0")/_colors.sh"

header "COMPLIANCE" "Built to survive an audit"
pause 1

# ── Audit table ──
subheader "Audit Table" "18 columns, 17 indexes — structured for compliance"
pause 0.3

type_cmd "systemprompt infra db describe governance_decisions"
pause 0.3
"$CLI" infra db describe governance_decisions --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -18 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1.2

divider

# ── Identity binding ──
subheader "Identity Binding" "user_id + session_id + trace_id on every decision"
pause 0.3

type_cmd "systemprompt infra db query \"SELECT user_id, session_id, decision, tool_name FROM governance_decisions ORDER BY created_at DESC LIMIT 4\""
pause 0.3
"$CLI" infra db query \
  "SELECT user_id, session_id, decision, tool_name FROM governance_decisions ORDER BY created_at DESC LIMIT 4" \
  --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1.2

divider

# ── Log retention ──
subheader "Structured Logs" "leveled, queryable, SIEM-ready"
pause 0.3

type_cmd "systemprompt infra logs summary --since 24h"
pause 0.3
"$CLI" infra logs summary --since 24h --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
check "identity-bound audit trail, structured logs, config-validated"
pause 1.5
