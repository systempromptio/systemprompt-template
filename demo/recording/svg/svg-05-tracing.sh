#!/bin/bash
# SVG RECORDING: Request Tracing
# Typed request flowing through 6 stages.
set -e
source "$(dirname "$0")/_colors.sh"

SESSION="svg-trace-$(date +%s)"

header "REQUEST TRACING" "End-to-end typed request flow"
pause 1

# ── Send a governance request ──
subheader "Governance request" "POST /api/public/hooks/govern"
pause 0.5

type_cmd "systemprompt hooks govern --agent developer_agent --tool Read"
pause 0.3

RESPONSE=$(curl -s -w '\n%{http_code} %{time_total}' -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\",\"agent_id\":\"developer_agent\",\"session_id\":\"$SESSION\",\"tool_input\":{\"file_path\":\"/src/main.rs\"}}" 2>/dev/null)

HTTP_CODE=$(echo "$RESPONSE" | tail -1 | awk '{print $1}')
HTTP_TIME=$(echo "$RESPONSE" | tail -1 | awk '{printf "%.1f", $2 * 1000}')
JSON_BODY=$(echo "$RESPONSE" | head -n -1)

echo "$JSON_BODY" | color_json
echo ""
echo -e "  ${GREEN}HTTP ${HTTP_CODE}${RESET} in ${CYAN}${HTTP_TIME}ms${RESET}"
pause 2

divider

# ── Request flow diagram ──
subheader "Request Pipeline" "6 typed stages, zero unstructured data"
echo ""
sleep 0.5
echo -e "    ${DIM}Client${RESET}"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${CYAN}▶ Axum Router${RESET}          POST /api/public/hooks/govern"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${CYAN}▶ JWT Validation${RESET}       extract_bearer → validate_jwt"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${CYAN}▶ Scope Resolution${RESET}     agent_id → admin | user"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${CYAN}▶ Rule Engine${RESET}          scope + secrets + rate_limit"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${YELLOW}▶ Async Audit${RESET}         tokio::spawn → INSERT"
echo -e "    ${DIM}  │${RESET}"
sleep 0.4
echo -e "    ${GREEN}▶ Response${RESET}            HTTP 200 { allow | deny }"
pause 2

divider

# ── Show trace ──
subheader "Execution Trace" "Every event logged with typed IDs"
pause 0.5

type_cmd "systemprompt infra logs trace list --limit 3"
pause 0.3

"$CLI" infra logs trace list --limit 3 --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -20 | while IFS= read -r line; do
  echo -e "    ${CYAN}${line}${RESET}"
done

pause 2

echo ""
echo -e "  ${CYAN}${BOLD}Every field typed. Every query compile-time checked.${RESET}"
echo ""
pause 2
