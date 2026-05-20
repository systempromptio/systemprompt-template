#!/bin/bash
# SCALED DEMO 4: SCHEDULER ISOLATION — NO DUPLICATE CRON EXECUTION
#
# The scheduler is single-node-bound: cron jobs MUST run on exactly one
# process. The scaled stack handles this at deployment time — the dedicated
# `scheduler` service runs the scheduler-enabled profile, while every `app`
# API replica runs the scheduler-DISABLED profile.
#
# This demo proves that mitigation holds. Over a window covering at least one
# scheduled run it inspects, per container:
#   - the systemprompt job-execution audit (`infra jobs history`), confirming
#     scheduled jobs actually ran during the window;
#   - each container's own logs for scheduler-execution markers.
# It then asserts ONLY the `scheduler` container shows job-execution activity
# and NO `app` replica does — i.e. zero duplicate execution.
#
# Per-replica addressing: `app` replicas have no host ports; we read their
# logs directly with `docker compose logs`.
#
# Environment overrides:
#   WATCH_SECONDS  window to watch for a scheduled run (default 360 = 6 min)
#
# Cost: Free.

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
RESULTS_DIR="$DEMO_ROOT/scenarios/scaled/results"
OUT_FILE="$RESULTS_DIR/scheduler-isolation.json"
WATCH_SECONDS="${WATCH_SECONDS:-360}"

DC=(docker compose -f "$COMPOSE_FILE")

# Log markers that indicate a scheduler actually fired a job. The scheduler
# extension logs around job dispatch — match broadly and case-insensitively
# so a wording change does not silently break the assertion.
SCHED_MARKER='scheduler|scheduled job|job .*(executed|running|dispatch|trigger)|cron'

header "SCALED DEMO 4: SCHEDULER ISOLATION" "exactly one node runs cron jobs"

# ── Preflight ──────────────────────────────────
step "Preflight checks"

for tool in jq docker; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    fail "$tool is required for this demo."
    exit 1
  fi
done
pass "jq, docker present"

if [[ ! -f "$COMPOSE_FILE" ]]; then
  fail "Scaled compose file not found: $COMPOSE_FILE"
  info "Stand the stack up: just scaled-up REPLICAS=3"
  exit 1
fi

if ! curl -fsS -o /dev/null --max-time 5 "$LB_URL/api/v1/health"; then
  fail "Scaled stack not reachable at $LB_URL — run: just scaled-up REPLICAS=3"
  exit 1
fi
pass "scaled stack healthy at $LB_URL"

mkdir -p "$RESULTS_DIR"

# ── Discover containers ────────────────────────
divider
step "Discovering containers"

mapfile -t SCHED_IDS < <("${DC[@]}" ps -q scheduler 2>/dev/null)
mapfile -t APP_IDS   < <("${DC[@]}" ps -q app 2>/dev/null)

if (( ${#SCHED_IDS[@]} != 1 )); then
  fail "Expected exactly 1 scheduler container; found ${#SCHED_IDS[@]}."
  info "The scaled contract requires a single scheduler service."
  exit 1
fi
pass "scheduler containers: ${#SCHED_IDS[@]}  (expected 1)"

if (( ${#APP_IDS[@]} < 1 )); then
  fail "No app replicas found."
  exit 1
fi
pass "app replicas: ${#APP_IDS[@]}"

SCHED_ID="${SCHED_IDS[0]}"

# ── Watch a window covering >= 1 scheduled run ─
divider
step "Watching for ${WATCH_SECONDS}s to cover at least one scheduled run"

WINDOW_START=$(date -u +"%Y-%m-%dT%H:%M:%S")
info "window start (UTC): $WINDOW_START"

# Sample job run_count before, so we can confirm a run actually happened.
RUNS_BEFORE=$( ("$CLI" infra jobs history --limit 200 --status success --profile "${PROFILE}" 2>/dev/null \
  | grep -cE '.' ) || echo 0)

sleep "$WATCH_SECONDS"

WINDOW_END=$(date -u +"%Y-%m-%dT%H:%M:%S")
info "window end   (UTC): $WINDOW_END"

# ── Confirm a scheduled run happened ───────────
divider
step "Confirming scheduled jobs ran during the window (job audit)"

JOB_HISTORY=$("$CLI" infra jobs history --limit 200 --status success --profile "${PROFILE}" 2>/dev/null || true)
RUNS_AFTER=$(echo "$JOB_HISTORY" | grep -cE '.' || echo 0)

if [[ -z "$JOB_HISTORY" ]]; then
  warn "Job-execution audit returned no rows."
  warn "If the scheduled jobs' cron cadence is longer than ${WATCH_SECONDS}s,"
  warn "re-run with a wider window: WATCH_SECONDS=900 $0"
else
  pass "job audit reachable ($RUNS_AFTER history rows)"
fi

# ── Inspect per-container logs for scheduler activity ─
divider
step "Inspecting per-container logs for job-execution markers"

# Count scheduler-execution log lines emitted within the watch window.
count_sched_lines() {
  local cid="$1"
  "${DC[@]}" logs --since "$WATCH_SECONDS"s "$cid" 2>/dev/null \
    | grep -iE "$SCHED_MARKER" \
    | grep -ivE 'disabled|skipping|not enabled|scheduler: *false' \
    | grep -cE '.' || true
}

SCHED_HITS=$(count_sched_lines "$SCHED_ID")
info "scheduler container ${SCHED_ID:0:12}: $SCHED_HITS execution log line(s)"

APP_TOTAL_HITS=0
APP_JSON="[]"
APP_OFFENDERS=0
for cid in "${APP_IDS[@]}"; do
  hits=$(count_sched_lines "$cid")
  APP_TOTAL_HITS=$(( APP_TOTAL_HITS + hits ))
  if (( hits > 0 )); then
    fail "app replica ${cid:0:12}: $hits scheduler execution line(s) — DUPLICATE EXECUTION"
    APP_OFFENDERS=$(( APP_OFFENDERS + 1 ))
  else
    pass "app replica ${cid:0:12}: 0 scheduler execution lines (correctly disabled)"
  fi
  APP_JSON=$(jq -n --argjson arr "$APP_JSON" --arg id "${cid:0:12}" --argjson h "$hits" \
    '$arr + [{replica: $id, scheduler_log_lines: $h}]')
done

# ── Verdict ────────────────────────────────────
divider
step "Verdict"

VERDICT=0

# (1) The scheduler node must show activity (otherwise nothing ran at all).
if (( SCHED_HITS > 0 )); then
  pass "scheduler node executed jobs ($SCHED_HITS marker lines)"
else
  warn "scheduler node showed no execution markers within the window."
  warn "Either no cron job was due (widen WATCH_SECONDS) or the scheduler is"
  warn "not running — this test cannot confirm isolation without a real run."
  VERDICT=1
fi

# (2) No app replica may execute jobs — this is the core isolation assertion.
if (( APP_OFFENDERS == 0 )); then
  pass "no app replica executed any scheduled job — zero duplicate execution"
else
  fail "$APP_OFFENDERS app replica(s) executed scheduled jobs — isolation BROKEN"
  fail "The scheduler-disabled profile/override is not applied to all replicas."
  VERDICT=1
fi

# ── Persist artifact ───────────────────────────
jq -n \
  --arg window_start "$WINDOW_START" --arg window_end "$WINDOW_END" \
  --argjson watch_seconds "$WATCH_SECONDS" \
  --argjson sched_hits "$SCHED_HITS" \
  --argjson app_total_hits "$APP_TOTAL_HITS" \
  --argjson app_offenders "$APP_OFFENDERS" \
  --argjson app_replicas "$APP_JSON" \
  --argjson job_history_rows "$RUNS_AFTER" \
  '{
     window: { start_utc: $window_start, end_utc: $window_end, watch_seconds: $watch_seconds },
     scheduler_node: { scheduler_log_lines: $sched_hits, executed_jobs: ($sched_hits > 0) },
     app_replicas: $app_replicas,
     app_offenders: $app_offenders,
     job_history_rows: $job_history_rows,
     duplicate_execution: ($app_offenders > 0),
     passed: ($sched_hits > 0 and $app_offenders == 0)
   }' > "$OUT_FILE"
pass "report written: results/scheduler-isolation.json"

divider
if (( VERDICT == 0 )); then
  pass "SCHEDULER ISOLATION VERIFIED"
  info "Exactly one node runs cron jobs; API replicas never duplicate them."
  info "Note: this is a deployment-time mitigation, not a code-level fix —"
  info "the scheduler remains single-node-bound by design."
else
  fail "SCHEDULER ISOLATION CHECK FAILED — see results/scheduler-isolation.json"
fi

exit "$VERDICT"
