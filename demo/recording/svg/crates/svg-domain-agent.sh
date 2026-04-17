#!/bin/bash
# domain-agent — agent runtime
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-agent" "Agents wired from flat YAML with schema validation"
pause 0.8

type_cmd "systemprompt admin agents validate"
pause 0.2
"$CLI" admin agents validate --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt admin agents show developer_agent"
pause 0.2
"$CLI" admin agents show developer_agent --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -20 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "agents as config, validated at load time"
pause 1.2
