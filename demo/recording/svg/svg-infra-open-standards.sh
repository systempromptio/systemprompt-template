#!/bin/bash
# SVG RECORDING: No Vendor Lock-In
# Every layer, an open standard. Zero proprietary protocols.
set -e
source "$(dirname "$0")/_colors.sh"

header "OPEN STANDARDS" "Every layer, an open standard"
pause 1

# ── MCP ──
subheader "MCP" "Model Context Protocol — tool servers over JSON-RPC"
pause 0.3

type_cmd "systemprompt plugins mcp status"
pause 0.3
"$CLI" plugins mcp status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── OAuth + JWT ──
subheader "OAuth 2.0 + JWT" "identity and authorization — open standards"
pause 0.3

type_cmd "systemprompt admin config security show"
pause 0.3
"$CLI" admin config security show --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

# ── PostgreSQL ──
subheader "PostgreSQL" "the only dependency — standard SQL, standard tooling"
pause 0.3

type_cmd "systemprompt infra db query \"SELECT decision, tool_name, agent_id FROM governance_decisions ORDER BY created_at DESC LIMIT 3\""
pause 0.3
"$CLI" infra db query \
  "SELECT decision, tool_name, agent_id FROM governance_decisions ORDER BY created_at DESC LIMIT 3" \
  --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""
check "MCP + OAuth 2.0 + JWT + PostgreSQL + YAML — zero proprietary protocols"
pause 1.5
