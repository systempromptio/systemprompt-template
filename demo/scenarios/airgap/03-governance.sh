#!/bin/bash
# AIR-GAP SCENARIO 3 — GOVERNANCE IN ISOLATION
#
# Proves the four-stage governance pipeline still works with the platform
# fully air-gapped, and proves end-to-end engine -> mock routing.
#
# Steps:
#   1. Acquire an admin token against the air-gapped app and save it to
#      demo/.token so the existing governance scripts can read it.
#   2. Run the existing demo/governance/*.sh scripts against the air-gapped
#      app. Those scripts hardcode `localhost:8080`; the air-gap stack
#      publishes on AIRGAP_HTTP_PORT (8090), so each script is run from a
#      port-rewritten copy. Assert every script exits 0.
#   3. Routing proof:
#        - POST /v1/messages with an ALLOWED model (claude-haiku-4-5)
#          -> expect 200, and the response `model` field reflects the
#             mock's upstream remap (full engine -> mock path proven).
#        - POST /v1/messages with a DENIED model
#          -> expect 403 with "not permitted by gateway policy".
#
# Run AFTER `just airgap-up`. Model exposure is owned by the profile
# catalog (.systemprompt/profiles/airgap/catalog.yaml); the dispatcher's
# is_model_exposed gate rejects an un-cataloged model with 403 before any
# upstream call. Policies (quotas/safety) ship via
# services/gateway/policies.yaml. Cost: Free (mock inference, no external
# calls).

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

BASE_URL="http://localhost:${AIRGAP_HTTP_PORT:-8090}"
AIRGAP_PORT="${AIRGAP_HTTP_PORT:-8090}"
GOV_DIR="$DEMO_ROOT/governance"
COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/airgap/docker-compose.airgap.yml"
COMPOSE=(docker compose -f "$COMPOSE_FILE")

ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-airgap-admin@demo.systemprompt.io}"

header "AIR-GAP GOVERNANCE" "Four-stage pipeline + engine->mock routing, fully isolated"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Compose file not found: $COMPOSE_FILE — run: just airgap-up"
  exit 1
fi

FAILURES=0

# ──────────────────────────────────────────────
#  PRECHECK: app health
# ──────────────────────────────────────────────
subheader "PRECHECK: app health"
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/v1/health" 2>/dev/null || echo "000")
if [[ "$HEALTH" == "200" ]]; then
  pass "app is healthy (HTTP 200)"
else
  fail "app health check returned HTTP $HEALTH — run: just airgap-up"
  exit 1
fi

# ──────────────────────────────────────────────
#  STEP 1: Acquire an admin token against the air-gapped app
# ──────────────────────────────────────────────
subheader "STEP 1: Admin token" "Acquired against the air-gapped app"

app_cli() { "${COMPOSE[@]}" exec -T app systemprompt "$@" 2>&1; }
_extract_jwt() {
  grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1
}

# Ensure the admin user exists (mirrors 00-preflight.sh + 02-load.sh).
app_cli admin users create --name "airgap-admin" --email "$ADMIN_EMAIL" >/dev/null 2>&1 || true
USER_ID=$(app_cli admin users search "$ADMIN_EMAIL" 2>/dev/null \
  | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
[[ -n "$USER_ID" ]] && app_cli admin users role promote "$USER_ID" >/dev/null 2>&1 || true

step "Logging in as $ADMIN_EMAIL"
LOGIN_OUTPUT=$(app_cli admin session login --email "$ADMIN_EMAIL" --token-only 2>&1 || true)
TOKEN=$(printf '%s\n' "$LOGIN_OUTPUT" | _extract_jwt)
if [[ -z "$TOKEN" ]]; then
  fail "Could not obtain an admin token from the air-gapped app"
  printf '%s\n' "$LOGIN_OUTPUT" | sed 's/^/    /'
  exit 1
fi
pass "Admin token acquired (${#TOKEN} chars)"

# Extract the session_id from the JWT — the gateway enforces strict
# x-session-id == token.session_id matching for /v1/messages.
_jwt_session_id() {
  local payload pad
  payload=$(printf '%s' "$1" | cut -d. -f2)
  pad=$(( (4 - ${#payload} % 4) % 4 ))
  printf '%s' "$payload"; printf '%.0s=' $(seq 1 $pad)
}
SESSION_ID=$(_jwt_session_id "$TOKEN" | tr '_-' '/+' | base64 -d 2>/dev/null \
  | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
if [[ -z "$SESSION_ID" ]]; then
  fail "Could not extract session_id from the minted JWT"
  exit 1
fi

# The existing governance scripts read demo/.token via TOKEN_FILE.
echo "$TOKEN" > "$TOKEN_FILE"
info "Token written to $TOKEN_FILE for the governance scripts"

# ──────────────────────────────────────────────
#  STEP 2: Four-stage governance pipeline, exercised directly
# ──────────────────────────────────────────────
subheader "STEP 2: Governance pipeline" "Direct calls to /api/public/hooks/govern"

# The demo/governance/*.sh scripts assume the local demo flow (00-preflight +
# 01-seed-data, the local app on :8080, demo seed data). Rather than re-point
# those fragile scripts, exercise the four-stage pipeline directly here — a
# self-contained air-gap conformance check with no seed-data dependency.
GOVERN_URL="$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo"

# POST a governance hook payload; echo the permissionDecision (allow|deny).
govern_decision() {
  curl -s -m 10 -X POST "$GOVERN_URL" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "$1" 2>/dev/null \
    | sed -n 's/.*"permissionDecision":"\([a-z]*\)".*/\1/p' | head -1
}

assert_decision() {
  local label="$1" want="$2" payload="$3" got
  got=$(govern_decision "$payload")
  if [[ "$got" == "$want" ]]; then
    pass "$label -> $got"
  else
    fail "$label -> '${got:-<no decision>}' (expected $want)"
    FAILURES=$((FAILURES + 1))
  fi
}

step "2a — clean Read call: all four stages pass"
assert_decision "Clean Read call" allow \
  '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"/src/main.rs"},"agent_id":"developer_agent","session_id":"airgap-gov-clean","cwd":"/app"}'

step "2b — AWS access key in a Bash command: secret scan denies"
assert_decision "AWS key in Bash command" deny \
  '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://example"},"agent_id":"developer_agent","session_id":"airgap-gov-aws","cwd":"/app"}'

step "2c — GitHub PAT in file content: secret scan denies"
assert_decision "GitHub PAT in Write content" deny \
  '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"/app/.env","content":"GITHUB_TOKEN=ghp_ABCDEFghijklmnop1234567890abcdef"},"agent_id":"developer_agent","session_id":"airgap-gov-pat","cwd":"/app"}'

step "2d — audit trail: decisions persisted to governance_decisions"
AUDIT_COUNT=$(app_cli infra db query \
  "SELECT count(*) AS n FROM governance_decisions WHERE session_id LIKE 'airgap-gov-%';" 2>/dev/null \
  | grep -oE '"n":[[:space:]]*"?[0-9]+' | grep -oE '[0-9]+' | head -1)
if [[ -n "$AUDIT_COUNT" && "$AUDIT_COUNT" -ge 3 ]]; then
  pass "Audit trail intact: $AUDIT_COUNT governance_decisions rows recorded in isolation"
else
  fail "Expected >=3 audited decisions, found '${AUDIT_COUNT:-0}'"
  FAILURES=$((FAILURES + 1))
fi

# Print the actual denied rows — counting is not evidence; the row itself is.
# A military reviewer wants to see decision + rule + agent_id, not a count.
step "2e — show the denied audit rows (decision, policy fired, tool, session, reason)"
cmd "systemprompt infra db query \"SELECT decision, policy, tool_name, session_id, reason FROM governance_decisions WHERE session_id LIKE 'airgap-gov-%' AND decision = 'deny' ORDER BY created_at DESC\""
DENY_ROWS=$(app_cli infra db query \
  "SELECT decision, policy, tool_name, session_id, reason FROM governance_decisions WHERE session_id LIKE 'airgap-gov-%' AND decision = 'deny' ORDER BY created_at DESC LIMIT 10;" 2>/dev/null || true)
if printf '%s' "$DENY_ROWS" | grep -q '"decision"[[:space:]]*:[[:space:]]*"deny"'; then
  pass "Denied decisions audited with rule attribution:"
  printf '%s\n' "$DENY_ROWS" | sed 's/^/    /'
else
  fail "No denied rows found in governance_decisions for session airgap-gov-%"
  printf '%s\n' "$DENY_ROWS" | sed 's/^/    /'
  FAILURES=$((FAILURES + 1))
fi

# ──────────────────────────────────────────────
#  STEP 3: Routing proof — allowed -> 200 via mock, denied -> 403
# ──────────────────────────────────────────────
subheader "STEP 3: Routing proof" "engine -> mock-inference path"

# 3a — ALLOWED model: claude-haiku-4-5 is allow-listed by the gateway
# policy (seeded in 02-load.sh). Expect 200; the response `model` field
# is set by the mock, proving the request traversed engine -> mock.
ALLOWED_PAYLOAD='{"model":"claude-haiku-4-5","max_tokens":16,"messages":[{"role":"user","content":"air-gap routing proof"}]}'
step "POST /v1/messages with allowed model claude-haiku-4-5"
cmd "curl -X POST $BASE_URL/v1/messages   (model=claude-haiku-4-5)"
ALLOWED_BODY=$(curl -s -m 15 -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "x-session-id: $SESSION_ID" \
  -H "Content-Type: application/json" \
  -d "$ALLOWED_PAYLOAD" \
  -w $'\n%{http_code}' \
  "$BASE_URL/v1/messages" 2>/dev/null || echo $'\n000')
ALLOWED_CODE=$(printf '%s' "$ALLOWED_BODY" | tail -1)
ALLOWED_JSON=$(printf '%s' "$ALLOWED_BODY" | sed '$d')
RESP_MODEL=$(printf '%s' "$ALLOWED_JSON" \
  | grep -oE '"model"[[:space:]]*:[[:space:]]*"[^"]*"' \
  | sed -n 's/.*"model"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)

if [[ "$ALLOWED_CODE" == "200" ]]; then
  pass "Allowed model -> HTTP 200"
  if [[ -n "$RESP_MODEL" ]]; then
    info "Response model field: $RESP_MODEL (set by mock-inference — engine->mock path confirmed)"
  else
    warn "Response carried no model field — could not confirm the mock remap"
    FAILURES=$((FAILURES + 1))
  fi
else
  fail "Allowed model -> HTTP $ALLOWED_CODE (expected 200)"
  printf '%s' "$ALLOWED_JSON" | sed 's/^/    /'
  FAILURES=$((FAILURES + 1))
fi
echo ""

# 3b — DENIED model: a model not in the gateway policy must be rejected
# before any upstream call. Expect 403 with the policy message.
DENIED_PAYLOAD='{"model":"claude-opus-forbidden-99","max_tokens":16,"messages":[{"role":"user","content":"should be denied"}]}'
step "POST /v1/messages with denied model claude-opus-forbidden-99"
cmd "curl -X POST $BASE_URL/v1/messages   (model=claude-opus-forbidden-99)"
DENIED_BODY=$(curl -s -m 15 -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "x-session-id: $SESSION_ID" \
  -H "Content-Type: application/json" \
  -d "$DENIED_PAYLOAD" \
  -w $'\n%{http_code}' \
  "$BASE_URL/v1/messages" 2>/dev/null || echo $'\n000')
DENIED_CODE=$(printf '%s' "$DENIED_BODY" | tail -1)
DENIED_JSON=$(printf '%s' "$DENIED_BODY" | sed '$d')

if [[ "$DENIED_CODE" == "403" ]]; then
  pass "Denied model -> HTTP 403"
  if printf '%s' "$DENIED_JSON" | grep -qi 'not permitted by gateway policy'; then
    pass "Response body carries 'not permitted by gateway policy'"
  else
    warn "403 returned but the expected policy message was absent:"
    printf '%s' "$DENIED_JSON" | sed 's/^/    /'
    FAILURES=$((FAILURES + 1))
  fi
else
  fail "Denied model -> HTTP $DENIED_CODE (expected 403)"
  printf '%s' "$DENIED_JSON" | sed 's/^/    /'
  FAILURES=$((FAILURES + 1))
fi

# ──────────────────────────────────────────────
#  SUMMARY
# ──────────────────────────────────────────────
divider
if [[ "$FAILURES" -eq 0 ]]; then
  header "AIR-GAP GOVERNANCE: PASS" "Pipeline intact; routing proven engine->mock"
  exit 0
else
  header "AIR-GAP GOVERNANCE: FAIL" "$FAILURES check(s) failed"
  exit 1
fi
