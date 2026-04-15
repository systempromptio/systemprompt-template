#!/bin/bash
# domain-oauth — OAuth flows
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-oauth" "OAuth sessions bound to tenants and users"
pause 0.8

type_cmd "systemprompt cloud auth status"
pause 0.2
"$CLI" cloud auth status --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt admin session show"
pause 0.2
"$CLI" admin session show --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "auth once, identity flows everywhere"
pause 1.2
