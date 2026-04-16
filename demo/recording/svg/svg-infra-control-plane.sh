#!/bin/bash
# SVG RECORDING: Unified Control Plane
# One CLI, every domain — govern, observe, manage.
set -e
source "$(dirname "$0")/_colors.sh"

header "UNIFIED CONTROL PLANE" "One CLI. Every domain."
pause 1

# ── Domains ──
subheader "8 domains" "core, infra, admin, cloud, analytics, web, plugins, build"
pause 0.3

type_cmd "systemprompt --help"
pause 0.3
"$CLI" --help 2>&1 | grep -E "^\s+(core|infra|admin|cloud|analytics|web|plugins|build)\s" \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── Govern ──
subheader "Govern" "every tool call checked before execution"
pause 0.3

type_cmd "systemprompt hooks govern --agent developer_agent --tool Read"
pause 0.3
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"svg-control-plane","cwd":"/var/www/html/systemprompt-template","tool_input":{}}' 2>/dev/null \
  | color_json
echo ""
pass "governance pipeline — 4 rules evaluated in-process"
pause 1.2

divider

# ── Observe ──
subheader "Observe" "analytics across every dimension"
pause 0.3

type_cmd "systemprompt analytics overview"
pause 0.3
"$CLI" analytics overview --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── Manage ──
subheader "Manage" "agents, users, config — all from the same CLI"
pause 0.3

type_cmd "systemprompt admin agents list"
pause 0.3
"$CLI" admin agents list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
check "govern + observe + manage — one binary, one CLI"
pause 1.5
