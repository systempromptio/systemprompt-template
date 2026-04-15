#!/bin/bash
# systemprompt-cli — top-level CLI surface
set -e
source "$(dirname "$0")/../_colors.sh"

header "systemprompt-cli" "One binary, every domain reachable through clap"
pause 0.8

type_cmd "systemprompt --help"
pause 0.2
"$CLI" --help 2>&1 | head -20 | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 0.8

type_cmd "systemprompt admin agents list"
pause 0.2
"$CLI" admin agents list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt core skills list"
pause 0.2
"$CLI" core skills list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "every domain in one binary, no sidecars"
pause 1.2
