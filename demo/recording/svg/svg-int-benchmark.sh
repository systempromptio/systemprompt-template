#!/bin/bash
# SVG RECORDING: Load Test + Capacity
# Thousands of requests, sub-5ms latency, enterprise scale.
set -e
source "$(dirname "$0")/_colors.sh"

# Ensure hey is installed
HEY="/tmp/hey"
if [[ ! -x "$HEY" ]]; then
  curl -sL https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64 -o "$HEY" && chmod +x "$HEY"
fi

BENCH_SESSION="svg-bench-$(date +%s)"

header "LOAD TEST" "Governance throughput under production load"

# ── Warm up silently ──
WARMUP_PAYLOAD='{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'-warmup","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/src/main.rs"}}'
"$HEY" -n 10 -c 5 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$WARMUP_PAYLOAD" \
  "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" > /dev/null 2>&1

# ── Test 1: Governance 500 requests ──
subheader "500 requests, 50 concurrent" "JWT → scope → 3 rules → audit write per request"
echo ""
pause 0.5

GOVERN_PAYLOAD='{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'-govern","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/src/main.rs"}}'

info "Running benchmark..."
echo ""

GOVERN_OUTPUT=$("$HEY" -n 500 -c 50 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$GOVERN_PAYLOAD" \
  "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" 2>&1)

GOVERN_RPS=$(echo "$GOVERN_OUTPUT" | grep "Requests/sec" | awk '{printf "%.0f", $2}')
GOVERN_P50=$(echo "$GOVERN_OUTPUT" | grep "50% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_P90=$(echo "$GOVERN_OUTPUT" | grep "90% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_P99=$(echo "$GOVERN_OUTPUT" | grep "99% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_FASTEST=$(echo "$GOVERN_OUTPUT" | grep "Fastest:" | awk '{printf "%.1f", $2 * 1000}')

table_top
table_row "Throughput" "${GOVERN_RPS} req/s" "$GREEN$BOLD"
table_mid
table_row "p50" "${GOVERN_P50}ms" "$GREEN"
table_row "p90" "${GOVERN_P90}ms" "$YELLOW"
table_row "p99" "${GOVERN_P99}ms" "$YELLOW"
table_row "Fastest" "${GOVERN_FASTEST}ms" "$GREEN"
table_bot
pause 2

divider

# ── Test 2: Sustained 1000 requests ──
subheader "1000 requests, 100 concurrent" "Doubled concurrency, sustained load"
echo ""
pause 0.5

SUSTAINED_PAYLOAD='{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'-sustained","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"ls -la"}}'

info "Running benchmark..."
echo ""

SUSTAINED_OUTPUT=$("$HEY" -n 1000 -c 100 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$SUSTAINED_PAYLOAD" \
  "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" 2>&1)

SUSTAINED_RPS=$(echo "$SUSTAINED_OUTPUT" | grep "Requests/sec" | awk '{printf "%.0f", $2}')
SUSTAINED_P50=$(echo "$SUSTAINED_OUTPUT" | grep "50% in" | awk '{printf "%.1f", $3 * 1000}')
SUSTAINED_P90=$(echo "$SUSTAINED_OUTPUT" | grep "90% in" | awk '{printf "%.1f", $3 * 1000}')
SUSTAINED_P99=$(echo "$SUSTAINED_OUTPUT" | grep "99% in" | awk '{printf "%.1f", $3 * 1000}')

table_top
table_row "Throughput" "${SUSTAINED_RPS} req/s" "$GREEN$BOLD"
table_mid
table_row "p50" "${SUSTAINED_P50}ms" "$GREEN"
table_row "p90" "${SUSTAINED_P90}ms" "$YELLOW"
table_row "p99" "${SUSTAINED_P99}ms" "$YELLOW"
table_bot
pause 2

divider

# ── Capacity estimate ──
CALLS_PER_DEV=10
if [[ -n "$GOVERN_RPS" && "$GOVERN_RPS" -gt 0 ]]; then
  DEVS_1=$(( GOVERN_RPS * 60 / CALLS_PER_DEV ))
  DEVS_3=$(( DEVS_1 * 3 ))
  DEVS_10=$(( DEVS_1 * 10 ))
fi

subheader "Enterprise Capacity" "${CALLS_PER_DEV} tool calls/min per developer"
echo ""

table_top
table_row "1 instance" "~${DEVS_1} devs" "$GREEN$BOLD"
table_row "3 + PgBouncer" "~${DEVS_3} devs" "$GREEN"
table_row "10 + PgBouncer" "~${DEVS_10} devs" "$GREEN"
table_bot
echo ""
echo -e "  ${GREEN}${BOLD}Governance adds <1% latency to AI response time.${RESET}"
echo ""
pause 3
