#!/bin/bash
# app-sync — cloud deployment status and runtime config backup
set -e
source "$(dirname "$0")/../_colors.sh"

header "app-sync" "Cloud deployment status and runtime config mirror"
pause 0.8

type_cmd "systemprompt cloud status"
pause 0.2
"$CLI" cloud status --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt cloud backup --list"
pause 0.2
"$CLI" cloud backup --list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "config is the contract, cloud is the mirror"
pause 1.2
