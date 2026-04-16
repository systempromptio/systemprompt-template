#!/bin/bash
# SVG RECORDING: Any AI Agent
# Any agent, any provider, one governance layer.
set -e
source "$(dirname "$0")/_colors.sh"

header "ANY AI AGENT" "Any agent. Any provider. One governance layer."
pause 1

# ── Agents ──
subheader "Agents" "admin and user scopes — different permissions, same governance"
pause 0.3

type_cmd "systemprompt admin agents list"
pause 0.3
"$CLI" admin agents list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

# ── Providers ──
subheader "Providers" "Anthropic, OpenAI, Gemini — swap at the profile level"
pause 0.3

type_cmd "systemprompt admin config provider list"
pause 0.3
"$CLI" admin config provider list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── Governance: admin → ALLOW ──
subheader "Admin agent" "developer_agent → admin scope"
pause 0.3

type_cmd "systemprompt hooks govern --agent developer_agent --tool list_agents"
pause 0.3
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"developer_agent","session_id":"svg-any-agent","cwd":"/var/www/html/systemprompt-template","tool_input":{}}' 2>/dev/null \
  | color_json
echo ""
pass "admin scope — all rules passed"
pause 1

divider

# ── Governance: user → DENY ──
subheader "User agent" "associate_agent → user scope"
pause 0.3

type_cmd "systemprompt hooks govern --agent associate_agent --tool list_agents"
pause 0.3
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"associate_agent","session_id":"svg-any-agent","cwd":"/var/www/html/systemprompt-template","tool_input":{}}' 2>/dev/null \
  | color_json
echo ""
fail "user scope — admin tool blocked"
pause 1

divider

echo -e "  ${CYAN}${BOLD}Any agent, any provider — one governance layer.${RESET}"
echo ""
pause 2
