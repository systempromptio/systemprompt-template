#!/bin/bash
# 00-overview — hero banner for core root README
set -e
source "$(dirname "$0")/../_colors.sh"

header "systemprompt" "One binary. Every domain. Library, not framework."
pause 0.8

type_cmd "systemprompt --help"
pause 0.2
"$CLI" --help 2>&1 | head -25 | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 0.6

type_cmd "systemprompt core --help | grep -E 'skills|content|files'"
pause 0.2
"$CLI" core --help --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | grep -E "skills|content|files|contexts|hooks" | head -5 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.4

type_cmd "systemprompt infra --help | grep -E 'services|db|logs'"
pause 0.2
"$CLI" infra --help --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | grep -E "services|db|logs|jobs" | head -5 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "core + infra + admin + cloud + analytics, one binary"
pause 1.2
