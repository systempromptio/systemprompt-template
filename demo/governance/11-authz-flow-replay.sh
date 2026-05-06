#!/bin/bash
# DEMO 11: AUTHZ FLOW REPLAY
# Subscribes to /admin/api/sse/audit in the background, fires 6 gateway
# requests in sequence (mixed allow/deny), kills the SSE, then renders an
# ASCII timeline from the captured events. Designed for screen-recording
# alongside the /admin/governance/flow page in the browser.
#
# Cost: Free for the authz path; allow requests do upstream calls.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

ADMIN_TOKEN="${TOKEN:-}"
if [[ -z "$ADMIN_TOKEN" && -f "$TOKEN_FILE" ]]; then
  ADMIN_TOKEN=$(cat "$TOKEN_FILE")
fi
if [[ -z "$ADMIN_TOKEN" ]]; then
  echo "ERROR: Run demo/00-preflight.sh first." >&2; exit 1
fi
if [[ ! -f /tmp/mint_jwt.py ]]; then
  fail "/tmp/mint_jwt.py missing — run demo 09 first."; exit 1
fi

ALICE_JWT=$(python3 /tmp/mint_jwt.py alice)
BOB_JWT=$(python3 /tmp/mint_jwt.py bob)
STREAM=/tmp/authz-stream.log

header "DEMO 11: AUTHZ FLOW REPLAY" "Live SSE capture + ASCII timeline + browser flow URL"

# ─────────────────────────────────────────────────
subheader "STEP 1: Subscribe to /admin/api/sse/audit (background)"

rm -f "$STREAM"
curl -sN --no-buffer "$BASE_URL/admin/api/sse/audit" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Accept: text/event-stream" >"$STREAM" 2>/dev/null &
SSE_PID=$!
sleep 1
if ! kill -0 "$SSE_PID" 2>/dev/null; then
  fail "SSE subscription failed to start"; exit 1
fi
pass "SSE subscriber pid=$SSE_PID"

# ─────────────────────────────────────────────────
subheader "STEP 2: Fire 6 gateway requests"

START_TS=$(date +%s.%N)
fire() {
  local who="$1" jwt="$2" model="$3"
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST "$BASE_URL/v1/messages" \
    -H "Authorization: Bearer $jwt" \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"$model\",\"max_tokens\":16,\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}")
  local ts
  ts=$(awk -v s="$START_TS" 'BEGIN{printf "%.2f", systime()-s}' 2>/dev/null \
        || python3 -c "import time;print(f'{time.time()-$START_TS:.2f}')")
  if [[ "$code" == "200" ]]; then
    printf "  T+%5ss  %-7s  %-15s  %s ALLOW%s\n" "$ts" "$who" "$model" "$GREEN" "$R"
  else
    printf "  T+%5ss  %-7s  %-15s  %s DENY %s (HTTP %s)\n" "$ts" "$who" "$model" "$RED" "$R" "$code"
  fi
}

fire alice "$ALICE_JWT" claude-3-sonnet ; sleep 0.5
fire alice "$ALICE_JWT" gpt-4            ; sleep 0.5
fire bob   "$BOB_JWT"   gpt-4            ; sleep 0.5
fire bob   "$BOB_JWT"   claude-3-sonnet  ; sleep 0.5
fire alice "$ALICE_JWT" claude-3-sonnet  ; sleep 0.5
fire bob   "$BOB_JWT"   gpt-4

# ─────────────────────────────────────────────────
subheader "STEP 3: Stop SSE subscriber"
sleep 1
kill "$SSE_PID" 2>/dev/null || true
wait "$SSE_PID" 2>/dev/null || true

EVENTS=$(grep -c '^event: audit' "$STREAM" 2>/dev/null || echo 0)
info "SSE events captured: $EVENTS"

# ─────────────────────────────────────────────────
subheader "STEP 4: Compact event timeline (governance_decisions only)"

awk '
  /^event: audit/ { getline data; sub(/^data: /,"",data); print data }
' "$STREAM" 2>/dev/null \
  | python3 -c '
import sys, json
for i, line in enumerate(sys.stdin):
    line = line.strip()
    if not line: continue
    try:
        ev = json.loads(line)
    except Exception:
        continue
    if ev.get("table") != "governance_decisions": continue
    sev = ev.get("severity","")
    mark = "✓" if sev == "info" else "✗"
    print(f"  {mark} {ev.get(\"policy\",\"?\"):>10}  {ev.get(\"decision\",\"?\"):>5}  {ev.get(\"tool_name\",\"\")}")
' || warn "could not render timeline (no jq/python parse failure)"

# ─────────────────────────────────────────────────
subheader "STEP 5: Open the flow page"
info "Browser view (same events, rendered live):"
echo "    $BASE_URL/admin/governance/flow?replay=last-5m"
echo ""

header "AUTHZ DEMO COMPLETE" "Bash output above ↔ flow timeline in browser"
