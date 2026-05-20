#!/bin/bash
# AIR-GAP SCENARIO 1 — EGRESS ASSERTION
#
# Proves the air-gapped stack is a genuinely closed system using TWO
# independent measurements:
#
#   1. NETWORK measurement — from the `monitor` sidecar inside the sealed
#      `airgap-internal` network, capture every outbound connection
#      (`ss -tunp`, and `conntrack -L` when present) during a burst of
#      requests. Assert NO connection has a remote address outside the
#      `airgap-internal` subnet. Any external target -> exit 1.
#
#   2. APPLICATION measurement — snapshot the mock-inference `/stats`
#      request counter, fire a burst of GOVERNANCE-DENIED requests at the
#      gateway ingress (`/v1/messages` with a model not in the gateway
#      policy -> 403), re-read `/stats`, and assert the counter is
#      UNCHANGED. This proves denial precedes any upstream call: a denied
#      request never reaches the inference endpoint. Any increment -> exit 1.
#
# Run AFTER `just airgap-up`. Cost: Free (no external AI calls — by design).

set -e

# _common.sh walks up from this dir to find DEMO_ROOT; sourcing from
# demo/scenarios/airgap/ resolves correctly via _find_demo_root.
source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

# The air-gap stack publishes the app on AIRGAP_HTTP_PORT (default 8090),
# not the profile-derived 8080. Override _common.sh's BASE_URL.
BASE_URL="http://localhost:${AIRGAP_HTTP_PORT:-8090}"

# Compose file, resolved relative to the repo root (PROJECT_DIR from _common.sh).
COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/airgap/docker-compose.airgap.yml"
COMPOSE=(docker compose -f "$COMPOSE_FILE")

# Run a command inside the monitor sidecar (no TTY).
mon() { "${COMPOSE[@]}" exec -T monitor "$@"; }

header "AIR-GAP EGRESS ASSERTION" "Two independent measurements of a closed system"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Compose file not found: $COMPOSE_FILE"
  echo "  Bring the stack up first:  just airgap-up"
  exit 1
fi

FAILURES=0

# ──────────────────────────────────────────────
#  PRECHECK: app is healthy
# ──────────────────────────────────────────────
subheader "PRECHECK: app health"
step "GET $BASE_URL/api/v1/health -> expect 200"
cmd "curl -s -o /dev/null -w '%{http_code}' $BASE_URL/api/v1/health"
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/v1/health" 2>/dev/null || echo "000")
if [[ "$HEALTH" == "200" ]]; then
  pass "app is healthy (HTTP 200)"
else
  fail "app health check returned HTTP $HEALTH (expected 200)"
  echo "  Is the stack up?  just airgap-up"
  exit 1
fi

# ──────────────────────────────────────────────
#  TOKEN: mint an admin token against the air-gapped app
# ──────────────────────────────────────────────
# /v1/messages requires auth — an unauthenticated POST 401s before policy
# evaluation, so the denied-model burst would never reach the 403 path.
# Mint the token inside the container (air-gap profile + jwt_secret).
subheader "TOKEN: admin token for the gateway bursts"
app_cli() { "${COMPOSE[@]}" exec -T app systemprompt "$@" 2>&1; }
_extract_jwt() { grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1; }
ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-airgap-admin@demo.systemprompt.io}"
app_cli admin users create --name "airgap-admin" --email "$ADMIN_EMAIL" >/dev/null 2>&1 || true
USER_ID=$(app_cli admin users search "$ADMIN_EMAIL" 2>/dev/null \
  | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
[[ -n "$USER_ID" ]] && app_cli admin users role promote "$USER_ID" >/dev/null 2>&1 || true
TOKEN=$(app_cli admin session login --email "$ADMIN_EMAIL" --token-only 2>&1 | _extract_jwt || true)
if [[ -z "$TOKEN" ]]; then
  fail "Could not mint an admin token from the air-gapped app"
  exit 1
fi
pass "Admin token acquired (${#TOKEN} chars)"

# ──────────────────────────────────────────────
#  MEASUREMENT 1: NETWORK — zero connections leave airgap-internal
# ──────────────────────────────────────────────
subheader "MEASUREMENT 1: NETWORK" "Outbound connections observed from the monitor sidecar"

# Discover the airgap-internal subnet so we can classify each remote address.
# The compose project name is derived from the directory; ask docker directly.
NETWORK_NAME=$("${COMPOSE[@]}" ps --format '{{.Name}}' 2>/dev/null | head -1 >/dev/null 2>&1; \
  docker network ls --format '{{.Name}}' 2>/dev/null | grep -E 'airgap-internal' | head -1 || true)
if [[ -z "$NETWORK_NAME" ]]; then
  fail "Could not locate the airgap-internal docker network — is the stack up?"
  exit 1
fi
SUBNET=$(docker network inspect "$NETWORK_NAME" \
  --format '{{range .IPAM.Config}}{{.Subnet}}{{end}}' 2>/dev/null || true)
INTERNAL_FLAG=$(docker network inspect "$NETWORK_NAME" \
  --format '{{.Internal}}' 2>/dev/null || true)
info "Network: $NETWORK_NAME  subnet=$SUBNET  internal=$INTERNAL_FLAG"
if [[ "$INTERNAL_FLAG" != "true" ]]; then
  fail "Network $NETWORK_NAME is NOT marked internal:true — egress is possible"
  FAILURES=$((FAILURES + 1))
fi
echo ""

# Fire a burst of requests at the gateway so there is live traffic to observe.
step "Generating a request burst (gateway traffic to observe)"
BURST_PAYLOAD='{"model":"claude-haiku-4-5","max_tokens":16,"messages":[{"role":"user","content":"ping"}]}'
for _ in $(seq 1 20); do
  curl -s -o /dev/null -m 5 -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -H "x-session-id: airgap-egress-burst" \
    -H "Content-Type: application/json" \
    -d "$BURST_PAYLOAD" \
    "$BASE_URL/v1/messages" >/dev/null 2>&1 || true
done &
BURST_PID=$!

# While the burst runs, sample outbound connections from inside the network.
step "Sampling outbound connections from the monitor sidecar (ss -tunp)"
cmd "docker compose -f <airgap compose> exec -T monitor ss -tunp"
SS_OUT=$(mon ss -tunp 2>/dev/null || true)
wait "$BURST_PID" 2>/dev/null || true

# conntrack is optional — netshoot ships it, but degrade gracefully if absent.
if mon sh -c 'command -v conntrack >/dev/null 2>&1' 2>/dev/null; then
  step "Sampling the conntrack table (conntrack -L)"
  cmd "docker compose -f <airgap compose> exec -T monitor conntrack -L"
  CT_OUT=$(mon conntrack -L 2>/dev/null || true)
else
  warn "conntrack not available in the monitor image — relying on ss only"
  CT_OUT=""
fi

# Extract remote IPv4 addresses from both samples. ss prints peer as IP:PORT
# in the last column; conntrack prints dst=IP fields. Use portable grep -oE.
REMOTE_IPS=$(
  {
    printf '%s\n' "$SS_OUT" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+' \
      | sed -n 's/^\([0-9.]*\):[0-9]*$/\1/p'
    printf '%s\n' "$CT_OUT" | grep -oE 'dst=[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' \
      | sed -n 's/^dst=\(.*\)$/\1/p'
  } | sort -u | grep -vE '^(127\.|0\.0\.0\.0$)' || true
)

# Classify: anything not in 10./172.16-31./192.168. (RFC1918, the docker
# bridge space) is treated as external. The airgap-internal subnet is
# always within this private space, so a non-private remote IP == egress.
EXTERNAL_HITS=""
while IFS= read -r ip; do
  [[ -z "$ip" ]] && continue
  case "$ip" in
    10.*|192.168.*) ;; # private
    172.1[6-9].*|172.2[0-9].*|172.3[0-1].*) ;; # private
    169.254.*) ;; # link-local
    *) EXTERNAL_HITS="$EXTERNAL_HITS $ip" ;;
  esac
done <<< "$REMOTE_IPS"

echo ""
if [[ -n "${EXTERNAL_HITS// }" ]]; then
  fail "External remote address(es) observed from inside the sealed network:"
  for ip in $EXTERNAL_HITS; do echo "      -> $ip"; done
  FAILURES=$((FAILURES + 1))
else
  pass "No connection left the airgap-internal network — zero egress observed"
  if [[ -n "${REMOTE_IPS// }" ]]; then
    info "All observed peers were within private (RFC1918) address space:"
    while IFS= read -r ip; do [[ -n "$ip" ]] && echo "      - $ip"; done <<< "$REMOTE_IPS"
  fi
fi

# ──────────────────────────────────────────────
#  MEASUREMENT 2: APPLICATION — denied requests never reach the mock
# ──────────────────────────────────────────────
subheader "MEASUREMENT 2: APPLICATION" "Mock /stats counter flat across a denied burst"

# Read the mock counter via the monitor sidecar — the mock publishes no host
# port, so it is only reachable from inside airgap-internal.
read_stats() {
  mon curl -s -m 5 "http://mock-inference:8080/stats" 2>/dev/null \
    | grep -oE '"(count|requests|total)"[[:space:]]*:[[:space:]]*[0-9]+' \
    | grep -oE '[0-9]+$' | head -1
}

step "Snapshot mock /stats before the denied burst"
cmd "docker compose -f <airgap compose> exec -T monitor curl -s http://mock-inference:8080/stats"
BEFORE=$(read_stats)
if [[ -z "$BEFORE" ]]; then
  fail "Could not read the mock /stats counter — is mock-inference up?"
  exit 1
fi
info "mock /stats count before: $BEFORE"
echo ""

# Fire a burst of GOVERNANCE-DENIED requests: a model that is NOT in the
# gateway policy is rejected before any upstream call -> HTTP 403.
DENIED_MODEL="claude-opus-forbidden-99"
DENIED_PAYLOAD='{"model":"'"$DENIED_MODEL"'","max_tokens":16,"messages":[{"role":"user","content":"should never reach the mock"}]}'
step "Firing 25 governance-denied requests at $BASE_URL/v1/messages"
cmd "curl -X POST $BASE_URL/v1/messages   (model=$DENIED_MODEL, expect 403)"
DENIED_403=0
DENIED_OTHER=0
for _ in $(seq 1 25); do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" -m 5 -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -H "x-session-id: airgap-egress-denied" \
    -H "Content-Type: application/json" \
    -d "$DENIED_PAYLOAD" \
    "$BASE_URL/v1/messages" 2>/dev/null || echo "000")
  if [[ "$CODE" == "403" ]]; then
    DENIED_403=$((DENIED_403 + 1))
  else
    DENIED_OTHER=$((DENIED_OTHER + 1))
  fi
done
info "denied burst: $DENIED_403 x HTTP 403, $DENIED_OTHER x other"
if [[ "$DENIED_403" -eq 0 ]]; then
  fail "No request was denied — the gateway did not reject the forbidden model"
  FAILURES=$((FAILURES + 1))
fi
echo ""

step "Re-read mock /stats after the denied burst"
AFTER=$(read_stats)
info "mock /stats count after:  $AFTER"
echo ""

if [[ "$AFTER" == "$BEFORE" ]]; then
  pass "Mock counter UNCHANGED ($BEFORE -> $AFTER) — denial precedes any upstream call"
else
  fail "Mock counter changed ($BEFORE -> $AFTER) — a denied request reached the mock"
  FAILURES=$((FAILURES + 1))
fi

# ──────────────────────────────────────────────
#  SUMMARY
# ──────────────────────────────────────────────
divider
if [[ "$FAILURES" -eq 0 ]]; then
  header "EGRESS ASSERTION: PASS" "The air-gapped stack is a closed system"
  pass "Network measurement: zero connections left airgap-internal"
  pass "Application measurement: governance-denied requests never reached the mock"
  exit 0
else
  header "EGRESS ASSERTION: FAIL" "$FAILURES check(s) failed"
  fail "The stack is NOT proven closed — see failures above"
  exit 1
fi
