#!/bin/bash
# domain-files — file storage
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-files" "Content-addressed file storage under storage/files/"
pause 0.8

type_cmd "systemprompt core files list"
pause 0.2
"$CLI" core files list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt core files stats"
pause 0.2
"$CLI" core files stats --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "files tracked, hashed, and queryable"
pause 1.2
