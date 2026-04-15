#!/bin/bash
# domain-users — user management
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-users" "Users, roles, bans — all from the CLI"
pause 0.8

type_cmd "systemprompt admin users list"
pause 0.2
"$CLI" admin users list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt admin users ban list"
pause 0.2
"$CLI" admin users ban list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "identity management without a separate IAM"
pause 1.2
