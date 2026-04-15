#!/bin/bash
# SVG RECORDING: Infrastructure
# One binary, one database, one CLI. Everything observable.
set -e
source "$(dirname "$0")/_colors.sh"

header "INFRASTRUCTURE" "One binary. One database. One CLI."
pause 1

# ── Services ──
subheader "Running services" "all in a single process — no sidecars"
pause 0.3

type_cmd "systemprompt infra services status"
pause 0.3

"$CLI" infra services status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pass "governance, agents, MCP, API, admin — all in-process"
pause 1.2

divider

# ── Database ──
subheader "Database" "the only runtime dependency"
pause 0.3

type_cmd "systemprompt infra db status"
pause 0.3

"$CLI" infra db status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -10 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
pause 1.2

divider

# ── Error-free logs ──
subheader "Logs" "structured, queryable, SIEM-ready"
pause 0.3

type_cmd "systemprompt infra logs view --since 1h --limit 8"
pause 0.3

"$CLI" infra logs view --since 1h --limit 8 --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1.5
