#!/bin/bash
# entry-api — HTTP surface
set -e
source "$(dirname "$0")/../_colors.sh"

header "entry-api" "REST + governance hooks over HTTP"
pause 0.8

type_cmd "curl -s \$BASE_URL/api/v1/agents | head -30"
pause 0.2
curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/api/v1/agents" 2>&1 | head -30 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.8

type_cmd "curl -sX POST \$BASE_URL/api/public/hooks/govern -d '{\"tool\":\"bash\",...}'"
pause 0.2
curl -s -X POST "$BASE_URL/api/public/hooks/govern" \
  -H "Content-Type: application/json" \
  -d '{"tool":"bash","input":{"command":"ls"}}' 2>&1 | head -20 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "every domain reachable over HTTP, governance on the edge"
pause 1.2
