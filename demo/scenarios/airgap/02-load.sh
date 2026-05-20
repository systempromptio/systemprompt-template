#!/bin/bash
# AIR-GAP SCENARIO 2 — LOAD TEST
#
# Drives the air-gapped gateway with the core loadtest harness and checks
# the airgap performance thresholds, all inside the sealed network.
#
# Steps:
#   1. Verify the gateway policy — the `ai_gateway_policies` row that
#      allow-lists `claude-haiku-4-5` is now ingested from
#      services/ai/gateway-policies.yaml by the publish_pipeline job at
#      server boot (see scenario architecture.md). This step only confirms
#      it landed; if it did not, ingestion failed and the run aborts.
#   2. Ensure the demo admin user exists (create + promote, mirroring
#      00-preflight.sh) and export SYSTEMPROMPT_ADMIN_EMAIL so the loadtest
#      self-acquires a token on the cloud-less air-gap profile.
#   3. Run the loadtest twice: --scenario gateway-inference, then
#      --scenario governance-only. JSON results land in results/.
#   4. Parse the JSON, check thresholds (governance p95 <= 300ms,
#      error rate <= 0.5%); fail + exit 1 on any breach.
#   5. Read the mock /stats counter and assert gateway-inference produced
#      ~1 mock hit per request while governance-only added 0 hits.
#
# Run AFTER `just airgap-up`. Cost: Free (mock inference — no external calls).

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

BASE_URL="http://localhost:${AIRGAP_HTTP_PORT:-8090}"
AIRGAP_DIR="$PROJECT_DIR/demo/scenarios/airgap"
RESULTS_DIR="$AIRGAP_DIR/results"
COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/airgap/docker-compose.airgap.yml"
COMPOSE=(docker compose -f "$COMPOSE_FILE")
CORE_DIR="$PROJECT_DIR/../systemprompt-core"
LOADTEST_MANIFEST="$CORE_DIR/crates/tests/loadtest/Cargo.toml"

# Demo admin email used for token self-acquisition by the loadtest.
ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-airgap-admin@demo.systemprompt.io}"

# Airgap thresholds (frozen contract). error_rate is a decimal fraction,
# so 0.5% == 0.005.
GOV_P95_MAX_MS=300
ERROR_RATE_MAX=0.005

mkdir -p "$RESULTS_DIR"

header "AIR-GAP LOAD TEST" "Core loadtest harness against the sealed gateway"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Compose file not found: $COMPOSE_FILE — run: just airgap-up"
  exit 1
fi
if [[ ! -f "$LOADTEST_MANIFEST" ]]; then
  fail "Loadtest crate not found at $LOADTEST_MANIFEST"
  echo "  Check out the core repo as a sibling:  ../systemprompt-core"
  exit 1
fi

# CLI runs against the air-gap database. The air-gap Postgres publishes no
# host port, so reach it through the `app` container — it carries the
# systemprompt CLI binary and DATABASE_URL pointing at the in-network DB.
db_exec() {
  "${COMPOSE[@]}" exec -T app systemprompt infra db execute "$1" 2>&1
}
db_query() {
  "${COMPOSE[@]}" exec -T app systemprompt infra db query "$1" 2>&1
}

FAILURES=0

# ──────────────────────────────────────────────
#  STEP 1: Verify the gateway policy
# ──────────────────────────────────────────────
subheader "STEP 1: Verify gateway policy" "claude-haiku-4-5 allow-listed (config-driven)"

# The gateway policy is no longer seeded by this script. It ships as
# version-controlled config — services/ai/gateway-policies.yaml — and the
# publish_pipeline job ingests it into ai_gateway_policies at server boot
# (run_gateway_policy_load), mirroring the access-control YAML loader. Here we
# only confirm the policy landed; if it did not, ingestion failed.
step "Confirming the gateway policy was ingested from YAML at server boot"
cmd "systemprompt infra db query \"SELECT spec FROM ai_gateway_policies WHERE enabled = true\""
POLICY_CHECK=$(db_query "SELECT spec FROM ai_gateway_policies WHERE enabled = true;" 2>/dev/null || true)
if printf '%s' "$POLICY_CHECK" | grep -q 'claude-haiku-4-5'; then
  pass "Confirmed: claude-haiku-4-5 is allow-listed in ai_gateway_policies"
else
  fail "claude-haiku-4-5 is NOT allow-listed — gateway-policy YAML ingestion did not run"
  echo "  Check services/ai/gateway-policies.yaml and the publish_pipeline job logs:"
  echo "    systemprompt infra logs view --level error --since 5m"
  exit 1
fi

# ──────────────────────────────────────────────
#  STEP 2: Admin user + token acquired inside the air-gap container
# ──────────────────────────────────────────────
subheader "STEP 2: Admin token" "Minted against the air-gap profile, passed to the loadtest"

app_cli() { "${COMPOSE[@]}" exec -T app systemprompt "$@" 2>&1; }
_extract_jwt() { grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1; }

step "Ensuring admin user $ADMIN_EMAIL exists"
# `users create` is idempotent for our purposes — a duplicate-email error
# just means the user already exists, which is success.
app_cli admin users create --name "airgap-admin" --email "$ADMIN_EMAIL" >/dev/null 2>&1 || true
USER_ID=$(app_cli admin users search "$ADMIN_EMAIL" 2>/dev/null \
  | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
if [[ -z "$USER_ID" ]]; then
  fail "Could not create or locate admin user $ADMIN_EMAIL"
  exit 1
fi
app_cli admin users role promote "$USER_ID" >/dev/null 2>&1 || true
pass "Admin user ready: $ADMIN_EMAIL ($USER_ID)"

# The loadtest's auth.rs::acquire_token hardcodes the *local* profile path
# and the local systemprompt binary — it cannot self-acquire against the
# air-gap profile (different DB + jwt_secret). So mint the token here, inside
# the container (air-gap profile, air-gap jwt_secret), and hand it to the
# loadtest via --token, which bypasses acquire_token entirely.
step "Minting admin token inside the air-gap container"
cmd "docker compose ... exec -T app systemprompt admin session login --email $ADMIN_EMAIL --token-only"
LOGIN_OUTPUT=$(app_cli admin session login --email "$ADMIN_EMAIL" --token-only 2>&1 || true)
LOAD_TOKEN=$(printf '%s\n' "$LOGIN_OUTPUT" | _extract_jwt)
if [[ -z "$LOAD_TOKEN" ]]; then
  fail "Could not mint an admin token from the air-gapped app"
  printf '%s\n' "$LOGIN_OUTPUT" | sed 's/^/    /'
  exit 1
fi
pass "Admin token minted (${#LOAD_TOKEN} chars) — will pass to the loadtest via --token"

# ──────────────────────────────────────────────
#  STEP 3: Run the loadtest twice
# ──────────────────────────────────────────────
subheader "STEP 3: Run loadtest" "gateway-inference, then governance-only"

GW_JSON="$RESULTS_DIR/load-gateway-inference.json"
GOV_JSON="$RESULTS_DIR/load-governance-only.json"

# The mock /stats counter is CUMULATIVE for the life of the container, so it
# must be measured as a before/after delta — not compared against an absolute.
read_stats() {
  "${COMPOSE[@]}" exec -T monitor curl -s -m 5 "http://mock-inference:8080/stats" 2>/dev/null \
    | grep -oE '"requests"[[:space:]]*:[[:space:]]*[0-9]+' \
    | grep -oE '[0-9]+' | tail -1
}

run_loadtest() {
  local scenario="$1" out_file="$2"
  step "Loadtest scenario: $scenario"
  cmd "cargo run --manifest-path ../systemprompt-core/crates/tests/loadtest/Cargo.toml -- --profile airgap --scenario $scenario --base-url $BASE_URL --token *** --output json --out-file $out_file"
  if ! cargo run --quiet --manifest-path "$LOADTEST_MANIFEST" -- \
      --profile airgap \
      --scenario "$scenario" \
      --base-url "$BASE_URL" \
      --token "$LOAD_TOKEN" \
      --output json \
      --out-file "$out_file"; then
    fail "Loadtest scenario '$scenario' exited non-zero"
    FAILURES=$((FAILURES + 1))
  fi
  if [[ ! -s "$out_file" ]]; then
    fail "Loadtest scenario '$scenario' produced no JSON at $out_file"
    FAILURES=$((FAILURES + 1))
  else
    pass "Wrote results: $out_file"
  fi
}

# Snapshot the cumulative mock counter around each run so STEP 5 can measure
# the delta attributable to each scenario.
STATS_BEFORE_GW=$(read_stats)
run_loadtest "gateway-inference" "$GW_JSON"
STATS_AFTER_GW=$(read_stats)
run_loadtest "governance-only" "$GOV_JSON"
STATS_AFTER_GOV=$(read_stats)

if [[ "$FAILURES" -ne 0 ]]; then
  divider
  header "AIR-GAP LOAD TEST: FAIL" "Loadtest runs did not complete"
  exit 1
fi

# ──────────────────────────────────────────────
#  STEP 4: Parse JSON, check thresholds
# ──────────────────────────────────────────────
subheader "STEP 4: Threshold checks" "governance p95 <= ${GOV_P95_MAX_MS}ms, error <= 0.5%"

if ! command -v jq >/dev/null 2>&1; then
  fail "jq is required to parse loadtest JSON results — install jq and re-run"
  exit 1
fi

# Loadtest JSON shape (core, reporters/json.rs):
#   { "scenarios": { "<name>": { "requests", "p50_ms", "p95_ms", "p99_ms",
#                                "error_rate", "passed" } },
#     "aggregate": { "total_requests", "all_passed" } }
# Each run is invoked with a single --scenario, so .scenarios has one entry.
# error_rate is a decimal fraction (0.0..1.0), NOT a percentage.
json_field() {
  # $1 = file, $2 = field within the (single) scenario object
  jq -r --arg f "$2" '(.scenarios | to_entries[0].value)[$f] // empty' "$1" 2>/dev/null || true
}

GW_REQS=$(json_field "$GW_JSON" requests)
GW_ERR=$(json_field "$GW_JSON" error_rate)

GOV_P95=$(json_field "$GOV_JSON" p95_ms)
GOV_ERR=$(json_field "$GOV_JSON" error_rate)
GOV_REQS=$(json_field "$GOV_JSON" requests)

# error_rate -> percentage for display.
pct() { awk -v r="$1" 'BEGIN{ if (r=="") print ""; else printf "%.3f", r*100 }'; }
info "gateway-inference: requests=$GW_REQS  error_rate=$(pct "$GW_ERR")%"
info "governance-only:   requests=$GOV_REQS  p95=${GOV_P95}ms  error_rate=$(pct "$GOV_ERR")%"
echo ""

# Floating-point comparison via awk (portable, no bc dependency).
fgt() { awk -v a="$1" -v b="$2" 'BEGIN{exit !(a>b)}'; }

if [[ -z "$GOV_P95" ]]; then
  fail "Could not read governance-only p95 latency from $GOV_JSON"
  FAILURES=$((FAILURES + 1))
elif fgt "$GOV_P95" "$GOV_P95_MAX_MS"; then
  fail "governance-only p95 ${GOV_P95}ms exceeds threshold ${GOV_P95_MAX_MS}ms"
  FAILURES=$((FAILURES + 1))
else
  pass "governance-only p95 ${GOV_P95}ms within ${GOV_P95_MAX_MS}ms"
fi

for pair in "gateway-inference:$GW_ERR" "governance-only:$GOV_ERR"; do
  name="${pair%%:*}"; rate="${pair##*:}"
  if [[ -z "$rate" ]]; then
    warn "$name: error rate not present in JSON — skipping that check"
  elif fgt "$rate" "$ERROR_RATE_MAX"; then
    fail "$name error rate $(pct "$rate")% exceeds threshold 0.5%"
    FAILURES=$((FAILURES + 1))
  else
    pass "$name error rate $(pct "$rate")% within 0.5%"
  fi
done

# ──────────────────────────────────────────────
#  STEP 5: Mock /stats correlation (measured as deltas)
# ──────────────────────────────────────────────
subheader "STEP 5: Mock /stats correlation" "gateway-inference hits the mock; governance-only does not"

step "Mock /stats counter, before/after each scenario"
info "before gateway-inference: ${STATS_BEFORE_GW:-?}   after: ${STATS_AFTER_GW:-?}   after governance-only: ${STATS_AFTER_GOV:-?}"
echo ""

if [[ -z "$STATS_BEFORE_GW" || -z "$STATS_AFTER_GW" || -z "$STATS_AFTER_GOV" ]]; then
  fail "Could not read the mock /stats counter (cumulative — measured as a delta)"
  FAILURES=$((FAILURES + 1))
else
  GW_HITS=$(( STATS_AFTER_GW - STATS_BEFORE_GW ))
  GOV_HITS=$(( STATS_AFTER_GOV - STATS_AFTER_GW ))
  info "gateway-inference added $GW_HITS mock hits; governance-only added $GOV_HITS"

  # gateway-inference: every allowed request reaches the mock (delta ~= requests).
  if [[ -n "$GW_REQS" ]]; then
    TOL=$(awk -v r="$GW_REQS" 'BEGIN{t=r*0.05; print (t<5?5:int(t))}')
    DELTA=$(awk -v a="$GW_HITS" -v b="$GW_REQS" 'BEGIN{d=a-b; print (d<0?-d:d)}')
    if awk -v d="$DELTA" -v t="$TOL" 'BEGIN{exit !(d<=t)}'; then
      pass "gateway-inference: $GW_HITS mock hits ~= $GW_REQS requests"
    else
      fail "gateway-inference mock hits ($GW_HITS) diverge from requests ($GW_REQS) by $DELTA (tolerance $TOL)"
      FAILURES=$((FAILURES + 1))
    fi
  fi

  # governance-only: every request is denied at the gateway -> ZERO mock hits.
  if [[ "$GOV_HITS" -eq 0 ]]; then
    pass "governance-only: 0 mock hits — denial precedes any upstream call"
  else
    fail "governance-only added $GOV_HITS mock hits — a denied request reached the mock"
    FAILURES=$((FAILURES + 1))
  fi
fi

# ──────────────────────────────────────────────
#  SUMMARY
# ──────────────────────────────────────────────
divider
if [[ "$FAILURES" -eq 0 ]]; then
  header "AIR-GAP LOAD TEST: PASS" "Thresholds met; mock correlation confirmed"
  pass "Results written to $RESULTS_DIR/"
  exit 0
else
  header "AIR-GAP LOAD TEST: FAIL" "$FAILURES check(s) failed"
  exit 1
fi
