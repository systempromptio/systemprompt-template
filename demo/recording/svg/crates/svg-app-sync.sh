#!/bin/bash
# app-sync — cloud sync
set -e
source "$(dirname "$0")/../_colors.sh"

header "app-sync" "Bidirectional config sync with the systemprompt cloud"
pause 0.8

type_cmd "systemprompt cloud sync status"
pause 0.2
"$CLI" cloud sync status --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt cloud sync pull --dry-run"
pause 0.2
"$CLI" cloud sync pull --dry-run --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "config is the contract, cloud is the mirror"
pause 1.2
