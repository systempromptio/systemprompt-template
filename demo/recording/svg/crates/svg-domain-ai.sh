#!/bin/bash
# domain-ai — AI request lifecycle
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-ai" "Every AI request captured, sanitized, auditable"
pause 0.8

type_cmd "systemprompt infra logs request list --limit 5"
pause 0.2
"$CLI" infra logs request list --limit 5 --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt infra logs request list --limit 1 --json"
pause 0.2
"$CLI" infra logs request list --limit 1 --json --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -20 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "every prompt, every tool call, fully replayable"
pause 1.2
