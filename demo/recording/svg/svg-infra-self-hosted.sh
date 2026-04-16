#!/bin/bash
# SVG RECORDING: Self-Hosted Deployment
# One binary. One process. One database. Full stack.
set -e
source "$(dirname "$0")/_colors.sh"

header "SELF-HOSTED DEPLOYMENT" "One binary. One process to audit."
pause 1

# ── Binary ──
subheader "Binary" "50MB of Rust — no runtime, no VM, no interpreter"
pause 0.3

type_cmd "systemprompt --version"
pause 0.3
"$CLI" --version 2>&1 | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── Process ──
subheader "Process" "agents, MCP servers, API — all in one process tree"
pause 0.3

type_cmd "systemprompt infra services status"
pause 0.3
"$CLI" infra services status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -8 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

# ── Database ──
subheader "Database" "PostgreSQL — the only runtime dependency"
pause 0.3

type_cmd "systemprompt infra db status"
pause 0.3
"$CLI" infra db status --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -6 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""
check "one binary, one process, one database — deploy anywhere"
pause 1.5
