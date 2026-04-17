#!/bin/bash
# domain-content — content ingestion
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-content" "Content sources ingested, normalized, searchable"
pause 0.8

type_cmd "systemprompt core content list --limit 10"
pause 0.2
"$CLI" core content list --limit 10 --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt core content sources list"
pause 0.2
"$CLI" core content sources list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "one pipeline from source to embedding"
pause 1.2
