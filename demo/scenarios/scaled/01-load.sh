#!/bin/bash
# SCALED DEMO 1: LOAD TEST THROUGH THE LOAD BALANCER
#
# Drives the Bucket 3 distributed load-test runner at
#   ../systemprompt-core/crates/tests/loadtest/
# against the scaled stack's nginx load balancer (http://localhost:8088).
#
# The runner spreads HTTP scenarios across whatever the LB fronts — with
# REPLICAS>1 that is N stateless `app` replicas round-robined by nginx.
#
# Asserts the `scaled` SLO from the plan:
#   p95 latency <= 500 ms   AND   error rate <= 2 %
#
# Output artifact: demo/scenarios/scaled/results/load.json
#
# Cost: Free (no AI inference scenarios are run).

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

# ── Frozen contract values ─────────────────────
COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
LOADTEST_MANIFEST="$PROJECT_DIR/../systemprompt-core/crates/tests/loadtest/Cargo.toml"
RESULTS_DIR="$DEMO_ROOT/scenarios/scaled/results"
OUT_FILE="$RESULTS_DIR/load.json"

# Thresholds (scaled SLO)
MAX_P95_MS=500
MAX_ERR_RATE=0.02

header "SCALED DEMO 1: LOAD TEST" "distributed runner -> nginx LB ($LB_URL)"

# ── Preflight ──────────────────────────────────
step "Preflight checks"

if ! command -v jq >/dev/null 2>&1; then
  fail "jq is required to parse the load-test JSON report."
  info "Install: brew install jq   (macOS)   |   sudo apt-get install -y jq   (Linux)"
  exit 1
fi
pass "jq present"

if ! command -v cargo >/dev/null 2>&1; then
  fail "cargo is required to run the load-test runner."
  exit 1
fi
pass "cargo present"

if [[ ! -f "$LOADTEST_MANIFEST" ]]; then
  fail "Load-test runner manifest not found:"
  info "  $LOADTEST_MANIFEST"
  info "Bucket 3 (../systemprompt-core) must be checked out alongside this repo."
  exit 1
fi
pass "load-test runner found"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Scaled compose file not found: $COMPOSE_FILE"
  info "Stand the stack up first: just scaled-up REPLICAS=3"
  exit 1
fi

# Confirm the LB is actually serving before we waste a load run on it.
if ! curl -fsS -o /dev/null --max-time 5 "$LB_URL/api/v1/health"; then
  fail "Scaled stack not reachable at $LB_URL"
  info "Stand it up first:  just scaled-up REPLICAS=3"
  exit 1
fi
pass "scaled stack healthy at $LB_URL"

mkdir -p "$RESULTS_DIR"

# ── Run the distributed load test ──────────────
divider
step "Running distributed load test against the LB"

# `--profile scaled` is the high-concurrency ramp (100 -> 1000 virtual users).
# `--scenario api-latency` exercises the unauthenticated public surface (/health
# + /.well-known/agent.json) — exactly what a misbehaving Claude Code client or
# k8s probe will hammer at scale, and the surface whose tail latency the LB
# must hold under load. It never touches the paid `gateway-inference`
# scenario, so this run costs nothing.
cmd "cargo run --manifest-path <loadtest>/Cargo.toml -- \\
      --profile scaled --scenario api-latency \\
      --base-url $LB_URL --output json --out-file results/load.json"

if ! cargo run --quiet --release --manifest-path "$LOADTEST_MANIFEST" -- \
      --profile scaled \
      --scenario api-latency \
      --base-url "$LB_URL" \
      --output json \
      --out-file "$OUT_FILE"; then
  # A non-zero exit means the runner's own thresholds failed — we still want
  # to read the JSON and report our scaled SLO verdict below, so don't abort.
  warn "Load-test runner exited non-zero (its internal thresholds were not met)."
fi

if [[ ! -f "$OUT_FILE" ]]; then
  fail "No JSON report was produced at $OUT_FILE"
  exit 1
fi
pass "report written: results/load.json"

# ── Assert the scaled SLO ──────────────────────
divider
step "Evaluating scaled SLO (p95 <= ${MAX_P95_MS}ms, error rate <= 2%)"

# Worst (max) p95 across all scenarios and the aggregate error rate.
WORST_P95=$(jq '[.scenarios[].p95_ms] | max' "$OUT_FILE")
TOTAL_REQ=$(jq '.aggregate.total_requests' "$OUT_FILE")
# Aggregate error rate: weighted is overkill — take the worst scenario rate.
WORST_ERR=$(jq '[.scenarios[].error_rate] | max' "$OUT_FILE")

info "total requests: $TOTAL_REQ"
info "worst p95:      ${WORST_P95}ms"
info "worst err rate: $(jq -n --argjson e "$WORST_ERR" '($e*100*100|round)/100')%"

VERDICT=0

if [[ "$WORST_P95" == "null" || -z "$WORST_P95" ]]; then
  fail "Could not read p95 from report — runner produced no scenario data."
  VERDICT=1
elif (( WORST_P95 <= MAX_P95_MS )); then
  pass "p95 ${WORST_P95}ms <= ${MAX_P95_MS}ms"
else
  fail "p95 ${WORST_P95}ms exceeds ${MAX_P95_MS}ms SLO"
  VERDICT=1
fi

# Float compare via jq (portable; no bc dependency).
if jq -e -n --argjson e "$WORST_ERR" --argjson m "$MAX_ERR_RATE" '$e <= $m' >/dev/null; then
  pass "error rate within 2% SLO"
else
  fail "error rate exceeds 2% SLO"
  VERDICT=1
fi

divider
if (( VERDICT == 0 )); then
  pass "SCALED LOAD SLO MET"
  info "Next: ./demo/scenarios/scaled/02-soak.sh"
else
  fail "SCALED LOAD SLO NOT MET — see results/load.json"
fi

exit "$VERDICT"
