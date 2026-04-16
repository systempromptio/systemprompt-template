#!/bin/bash
# SVG RECORDING: Governance Allow + Deny
set -e
source "$(dirname "$0")/_colors.sh"

header "GOVERNANCE" "Tool access control for AI agents"
sleep 0.8

subheader "Admin agent requests tool access" "developer_agent → mcp__systemprompt__list_agents"
sleep 0.3

type_cmd "systemprompt hooks govern --agent developer_agent --tool list_agents"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"developer_agent","session_id":"svg-governance","cwd":"/var/www/html/systemprompt-template","tool_input":{}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
pass "admin scope, all 3 rules passed"
sleep 1

divider

subheader "User agent requests same tool" "associate_agent → mcp__systemprompt__list_agents"
sleep 0.3

type_cmd "systemprompt hooks govern --agent associate_agent --tool list_agents"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"associate_agent","session_id":"svg-governance","cwd":"/var/www/html/systemprompt-template","tool_input":{}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
fail "scope_restriction — user cannot access admin tools"
sleep 1

divider

printf "${CYAN}${BOLD}Same tool. Two agents. Two outcomes. Both audited.${R}\n"
echo ""
sleep 1.5
