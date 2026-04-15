#!/bin/bash
# domain-templates — template rendering
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-templates" "Handlebars templates validated and rendered server-side"
pause 0.8

type_cmd "systemprompt web templates list"
pause 0.2
"$CLI" web templates list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt web validate"
pause 0.2
"$CLI" web validate --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "templates are code — validated, typed, tested"
pause 1.2
