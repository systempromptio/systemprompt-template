#!/bin/bash
# SCALED DEMO 2: SOAK TEST — SUSTAINED LOAD, DRIFT DETECTION
#
# Runs the load-test runner's native `soak` profile against the nginx LB — a
# single ~1 hour sustained run at steady concurrency — and asserts the service
# does not degrade over the window.
#
# Latency drift is read from the runner's own `time_series[]` (one window per
# `--sample-interval-secs`): the mean p95 of the last windows is compared to
# the mean p95 of the first windows. Memory drift is sampled independently:
# a background sampler sums resident memory across the `app` replicas while
# the run is in flight.
#
# A soak surfaces what a single burst hides: leaked memory, growing GC
# pressure, connection-pool exhaustion, slow p95 creep.
#
# Output artifact: demo/scenarios/scaled/results/soak.json
#   (the runner's full JSON report, augmented with a `soak_analysis` block)
#
# Environment overrides:
#   SAMPLE_INTERVAL  seconds per latency window, passed to the runner as
#                    --sample-interval-secs (default 30)
#   MEM_INTERVAL     seconds between memory samples (default 60)
#
# Cost: Free — only the `governance-only` scenario runs (no AI inference).

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
LOADTEST_MANIFEST="$PROJECT_DIR/../systemprompt-core/crates/tests/loadtest/Cargo.toml"
RESULTS_DIR="$DEMO_ROOT/scenarios/scaled/results"
OUT_FILE="$RESULTS_DIR/soak.json"
MEM_FILE="$RESULTS_DIR/.soak-memory.tsv"

SAMPLE_INTERVAL="${SAMPLE_INTERVAL:-30}"
MEM_INTERVAL="${MEM_INTERVAL:-60}"
MAX_DRIFT_PCT=5

DC=(docker compose -f "$COMPOSE_FILE")

header "SCALED DEMO 2: SOAK TEST" "native soak profile + drift detection (~1 h)"

# ── Preflight ──────────────────────────────────
step "Preflight checks"

for tool in jq cargo docker; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    fail "$tool is required for the soak test."
    exit 1
  fi
done
pass "jq, cargo, docker present"

if [[ ! -f "$LOADTEST_MANIFEST" ]]; then
  fail "Load-test runner manifest not found: $LOADTEST_MANIFEST"
  info "Bucket 3 (../systemprompt-core) must be checked out alongside this repo."
  exit 1
fi
pass "load-test runner found"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Scaled compose file not found: $COMPOSE_FILE"
  exit 1
fi

if ! curl -fsS -o /dev/null --max-time 5 "$LB_URL/api/v1/health"; then
  fail "Scaled stack not reachable at $LB_URL — run: just scaled-up REPLICAS=3"
  exit 1
fi
pass "scaled stack healthy at $LB_URL"

mkdir -p "$RESULTS_DIR"

# ── Helper: total resident memory (KiB) across app replicas ─────
replica_mem_kib() {
  local total=0 cid stat
  for cid in $("${DC[@]}" ps -q app 2>/dev/null); do
    # `docker stats` MemUsage looks like "123.4MiB / 1.95GiB" — take field 1.
    stat=$(docker stats --no-stream --format '{{.MemUsage}}' "$cid" 2>/dev/null | awk '{print $1}')
    total=$(awk -v s="$stat" -v acc="$total" '
      BEGIN {
        v = s; unit = s;
        sub(/[A-Za-z]+$/, "", v); sub(/^[0-9.]+/, "", unit);
        if (unit ~ /GiB/) v *= 1048576;
        else if (unit ~ /MiB/) v *= 1024;
        else if (unit ~ /KiB/) v *= 1;
        else if (unit ~ /B/) v /= 1024;
        printf "%.0f", acc + v;
      }')
  done
  echo "$total"
}

# ── Background memory sampler ──────────────────
divider
step "Starting background memory sampler (every ${MEM_INTERVAL}s)"

: > "$MEM_FILE"
(
  while true; do
    echo "$(date +%s)	$(replica_mem_kib)" >> "$MEM_FILE"
    sleep "$MEM_INTERVAL"
  done
) &
SAMPLER_PID=$!
# Make sure the sampler is reaped however the script exits.
trap 'kill "$SAMPLER_PID" 2>/dev/null || true' EXIT
pass "memory sampler running (pid $SAMPLER_PID)"

# ── Run the soak ───────────────────────────────
divider
step "Running native soak profile (governance-only, ~1 h) — be patient"

cmd "cargo run --manifest-path <loadtest>/Cargo.toml -- \\
      --profile soak --scenario api-latency \\
      --sample-interval-secs $SAMPLE_INTERVAL \\
      --base-url $LB_URL --output json --out-file results/soak.json"

# Non-zero exit means the runner's own thresholds failed — we still want to
# read the JSON and report our drift verdict, so don't abort here.
if ! cargo run --quiet --release --manifest-path "$LOADTEST_MANIFEST" -- \
      --profile soak \
      --scenario api-latency \
      --sample-interval-secs "$SAMPLE_INTERVAL" \
      --base-url "$LB_URL" \
      --output json \
      --out-file "$OUT_FILE"; then
  warn "Load-test runner exited non-zero (its internal thresholds were not met)."
fi

kill "$SAMPLER_PID" 2>/dev/null || true
trap - EXIT

if [[ ! -f "$OUT_FILE" ]]; then
  fail "No JSON report was produced at $OUT_FILE"
  exit 1
fi
pass "runner report written: results/soak.json"

# ── Latency drift from the runner's time series ─
divider
step "Latency drift analysis (time_series windows)"

# Compare the mean p95 of the first 10% of request-bearing windows to the
# mean p95 of the last 10%. Windows with zero requests (ramp-down tail) are
# excluded so the drift figure is not skewed by empty buckets.
LAT=$(jq '
  ([.scenarios[].time_series // []] | add // [])
  | map(select(.requests > 0))
  | if length == 0 then
      { error: "no time-series windows with requests" }
    else
      ( [1, (length / 10 | floor)] | max ) as $k
      | (.[0:$k]    | map(.p95_ms) | add / length) as $early
      | (.[-$k:]    | map(.p95_ms) | add / length) as $late
      | {
          windows: length,
          early_p95_ms: ($early | (.*100|round)/100),
          late_p95_ms:  ($late  | (.*100|round)/100),
          drift_pct: (if $early == 0 then 0
                      else (($late - $early) / $early * 100 * 100 | round) / 100 end)
        }
    end
' "$OUT_FILE")

if [[ "$(jq -r 'has("error")' <<<"$LAT")" == "true" ]]; then
  fail "$(jq -r '.error' <<<"$LAT") — cannot compute latency drift."
  info "Was --sample-interval-secs ($SAMPLE_INTERVAL) shorter than the run?"
  exit 1
fi

LAT_EARLY=$(jq -r '.early_p95_ms' <<<"$LAT")
LAT_LATE=$(jq -r '.late_p95_ms'  <<<"$LAT")
LAT_DRIFT=$(jq -r '.drift_pct'   <<<"$LAT")
LAT_WINDOWS=$(jq -r '.windows'   <<<"$LAT")
info "windows analysed: $LAT_WINDOWS"
info "p95: ${LAT_EARLY}ms (early) -> ${LAT_LATE}ms (late)  (drift ${LAT_DRIFT}%)"

# ── Memory drift from the background sampler ────
divider
step "Memory drift analysis (app-replica resident memory)"

MEM_SAMPLES=$(grep -c '' "$MEM_FILE" 2>/dev/null || echo 0)
if (( MEM_SAMPLES < 2 )); then
  fail "Memory sampler captured < 2 samples — cannot compute memory drift."
  exit 1
fi

BASE_MEM=$(head -n1 "$MEM_FILE" | cut -f2)
LAST_MEM=$(tail -n1 "$MEM_FILE" | cut -f2)
MEM_DRIFT=$(jq -n --argjson b "${BASE_MEM:-0}" --argjson l "${LAST_MEM:-0}" \
  'if $b == 0 then 0 else (($l - $b) / $b * 100 * 100 | round) / 100 end')
info "samples: $MEM_SAMPLES"
info "memory: ${BASE_MEM}KiB -> ${LAST_MEM}KiB  (drift ${MEM_DRIFT}%)"

# ── Persist merged artifact ────────────────────
TMP=$(mktemp)
jq \
  --argjson le "$LAT_EARLY" --argjson ll "$LAT_LATE" --argjson ld "$LAT_DRIFT" \
  --argjson lw "$LAT_WINDOWS" \
  --argjson bm "${BASE_MEM:-0}" --argjson fm "${LAST_MEM:-0}" --argjson md "$MEM_DRIFT" \
  --argjson ms "$MEM_SAMPLES" --argjson max "$MAX_DRIFT_PCT" \
  '. + {
     soak_analysis: {
       latency: { windows: $lw, early_p95_ms: $le, late_p95_ms: $ll, drift_pct: $ld },
       memory:  { samples: $ms, baseline_kib: $bm, final_kib: $fm, drift_pct: $md },
       max_drift_pct: $max,
       passed: (($ld | fabs) <= $max and ($md | fabs) <= $max)
     }
   }' "$OUT_FILE" > "$TMP" && mv "$TMP" "$OUT_FILE"
rm -f "$MEM_FILE"
pass "merged report written: results/soak.json"

# ── Verdict ────────────────────────────────────
divider
VERDICT=0

if jq -e -n --argjson d "$LAT_DRIFT" --argjson m "$MAX_DRIFT_PCT" '($d|fabs) <= $m' >/dev/null; then
  pass "latency drift within ${MAX_DRIFT_PCT}%"
else
  fail "latency drifted ${LAT_DRIFT}% (> ${MAX_DRIFT_PCT}%) — possible degradation under sustained load"
  VERDICT=1
fi

if jq -e -n --argjson d "$MEM_DRIFT" --argjson m "$MAX_DRIFT_PCT" '($d|fabs) <= $m' >/dev/null; then
  pass "memory drift within ${MAX_DRIFT_PCT}%"
else
  fail "memory drifted ${MEM_DRIFT}% (> ${MAX_DRIFT_PCT}%) — possible leak"
  VERDICT=1
fi

divider
if (( VERDICT == 0 )); then
  pass "SOAK PASSED — stable under sustained load"
else
  fail "SOAK FAILED — see results/soak.json"
fi

exit "$VERDICT"
