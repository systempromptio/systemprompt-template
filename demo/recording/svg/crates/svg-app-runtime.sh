#!/bin/bash
# app-runtime — service lifecycle
set -e
source "$(dirname "$0")/../_colors.sh"

header "app-runtime" "Supervises every long-running service in one process"
pause 0.8

type_cmd "systemprompt infra services list"
pause 0.2
"$CLI" infra services list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt infra services status"
pause 0.2
"$CLI" infra services status --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "one supervisor, every service observable from the CLI"
pause 1.2
