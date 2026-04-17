#!/bin/bash
# domain-analytics — usage analytics
set -e
source "$(dirname "$0")/../_colors.sh"

header "domain-analytics" "Usage, cost, and tool analytics in one query layer"
pause 0.8

type_cmd "systemprompt analytics agents show developer_agent"
pause 0.2
"$CLI" analytics agents show developer_agent --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt analytics costs breakdown --by agent"
pause 0.2
"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "every token accounted for, every agent measured"
pause 1.2
