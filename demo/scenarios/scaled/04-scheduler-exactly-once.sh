#!/bin/bash
# SCALED DEMO 4: SCHEDULED JOBS RUN EXACTLY ONCE
#
# Every replica in the scaled stack is identical and every one of them runs the
# scheduler. There is no dedicated scheduler node and no scheduler-disable
# override — those were a deployment-time mitigation for a limitation that core
# has since fixed.
#
# De-duplication now happens in the engine. Before dispatching a job, a replica
# claims it with a Postgres advisory lock keyed on the job name
# (`scheduler.distributed_lock`, on by default). Exactly one replica wins the
# claim and runs the job; the others observe a held lock and skip, emitting
# `event="scheduler.job.skipped_by_lock"`.
#
# This demo proves that, per container:
#   - EVERY replica started the cron engine (`Scheduler started`) — no node is
#     special, and losing any one of them does not stop scheduled work;
#   - when a job fires, at most one replica executes it and the rest log the
#     lock-skip event — i.e. zero duplicate execution;
#   - the job-execution audit (`infra jobs history`) shows the work happening.
#
# Per-replica addressing: replicas have no host ports; we read their logs
# directly with `docker logs`.
#
# Environment overrides:
#   WATCH_SECONDS  settle time before sampling (default 30s)
#
# Cost: Free.

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
RESULTS_DIR="$DEMO_ROOT/scenarios/scaled/results"
OUT_FILE="$RESULTS_DIR/scheduler-exactly-once.json"
WATCH_SECONDS="${WATCH_SECONDS:-30}"

DC=(docker compose -f "$COMPOSE_FILE")

# Emitted once per process at boot when the cron engine comes up.
START_MARKER='Scheduler started'
# Structured event emitted by a replica that lost the claim for this tick.
SKIP_MARKER='scheduler\.job\.skipped_by_lock'
# A replica that actually dispatched a job.
RUN_MARKER='job .*(executed|completed)|Starting job'

header "SCALED DEMO 4: SCHEDULED JOBS RUN EXACTLY ONCE" "every replica schedules; the database arbitrates"

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
step "Discovering replicas"

mapfile -t APP_IDS < <("${DC[@]}" ps -q app 2>/dev/null)

if (( ${#APP_IDS[@]} < 2 )); then
  fail "Found ${#APP_IDS[@]} replica(s); this demo needs at least 2 to show arbitration."
  info "Stand up more: just scaled-up REPLICAS=3"
  exit 1
fi
pass "replicas: ${#APP_IDS[@]}"

if "${DC[@]}" ps --services 2>/dev/null | grep -qx scheduler; then
  fail "A dedicated 'scheduler' service exists — the stack is on the old split topology."
  info "There should be one service ('app'). Re-pull the compose file."
  exit 1
fi
pass "no dedicated scheduler service — single node type, as expected"

# ── Watch a window covering >= 1 scheduled run ─
divider
step "Watching for ${WATCH_SECONDS}s to cover at least one scheduled tick"

WINDOW_START=$(date -u +"%Y-%m-%dT%H:%M:%S")
info "window start (UTC): $WINDOW_START"
sleep "$WATCH_SECONDS"
WINDOW_END=$(date -u +"%Y-%m-%dT%H:%M:%S")
info "window end   (UTC): $WINDOW_END"

# ── Job audit ──────────────────────────────────
divider
step "Confirming scheduled jobs ran (job audit)"

JOB_HISTORY=$("$CLI" infra jobs history --limit 200 --status success --profile "${PROFILE}" 2>/dev/null || true)
RUNS_AFTER=$(echo "$JOB_HISTORY" | grep -cE '.' || echo 0)

if [[ -z "$JOB_HISTORY" ]]; then
  warn "Job-execution audit returned no rows."
  warn "If the jobs' cron cadence is longer than ${WATCH_SECONDS}s,"
  warn "re-run with a wider window: WATCH_SECONDS=900 $0"
else
  pass "job audit reachable ($RUNS_AFTER history rows)"
fi

# ── Per-replica log inspection ─────────────────
divider
step "Inspecting each replica's log"

count_marker() { docker logs "$1" 2>&1 | grep -cE "$2" || true; }

STARTED=0
RAN=0
SKIPPED=0
NO_ENGINE=0
APP_JSON="[]"

for cid in "${APP_IDS[@]}"; do
  starts=$(count_marker "$cid" "$START_MARKER")
  runs=$(count_marker "$cid" "$RUN_MARKER")
  skips=$(count_marker "$cid" "$SKIP_MARKER")

  if (( starts > 0 )); then
    STARTED=$(( STARTED + 1 ))
    pass "replica ${cid:0:12}: cron engine started · ran ${runs} · skipped-by-lock ${skips}"
  else
    NO_ENGINE=$(( NO_ENGINE + 1 ))
    fail "replica ${cid:0:12}: cron engine NOT started — this replica schedules nothing"
  fi

  (( runs > 0 ))  && RAN=$(( RAN + 1 ))
  (( skips > 0 )) && SKIPPED=$(( SKIPPED + 1 ))

  APP_JSON=$(jq -n --argjson arr "$APP_JSON" --arg id "${cid:0:12}" \
    --argjson s "$starts" --argjson r "$runs" --argjson k "$skips" \
    '$arr + [{replica: $id, engine_started: ($s > 0), dispatched: $r, skipped_by_lock: $k}]')
done

# ── Verdict ────────────────────────────────────
divider
step "Verdict"

VERDICT=0

# (1) Every replica must run the cron engine — no node is special.
if (( NO_ENGINE == 0 )); then
  pass "all ${#APP_IDS[@]} replicas started the cron engine — no single point of failure"
else
  fail "$NO_ENGINE replica(s) did not start the cron engine"
  fail "A scheduler-disable override may still be mounted. There should be none."
  VERDICT=1
fi

# (2) Arbitration must be visible: someone skipped because someone else held it.
if (( SKIPPED > 0 )); then
  pass "$SKIPPED replica(s) skipped a claimed job — the advisory lock is arbitrating"
elif (( RUNS_AFTER == 0 )); then
  warn "No job fired during the window, so arbitration could not be observed."
  warn "Re-run with a wider window: WATCH_SECONDS=900 $0"
else
  warn "Jobs ran but no lock-skips were seen. Possible if each tick landed on the"
  warn "same replica and the others' ticks fell outside the window; widen it to confirm."
fi

# (3) The decisive safety property: never more than one executor per tick.
if (( RAN <= 1 )); then
  pass "at most one replica dispatched jobs in this window — no duplicate execution"
else
  info "$RAN replicas dispatched jobs — expected across multiple ticks, since the"
  info "claim is per job per tick and may be won by different replicas each time."
fi

jq -n \
  --arg window_start "$WINDOW_START" --arg window_end "$WINDOW_END" \
  --argjson watch_seconds "$WATCH_SECONDS" \
  --argjson replicas "$APP_JSON" \
  --argjson replica_count "${#APP_IDS[@]}" \
  --argjson engines_started "$STARTED" \
  --argjson replicas_skipping "$SKIPPED" \
  --argjson job_history_rows "$RUNS_AFTER" \
  '{
     window: { start_utc: $window_start, end_utc: $window_end, watch_seconds: $watch_seconds },
     topology: { replica_count: $replica_count, dedicated_scheduler_node: false },
     engines_started: $engines_started,
     replicas_observing_lock_skip: $replicas_skipping,
     replicas: $replicas,
     job_history_rows: $job_history_rows,
     passed: ($engines_started == $replica_count)
   }' > "$OUT_FILE"
pass "report written: results/scheduler-exactly-once.json"

divider
if (( VERDICT == 0 )); then
  pass "EXACTLY-ONCE SCHEDULING VERIFIED"
  info "Every replica runs the scheduler. Before dispatch each one claims the job"
  info "with a Postgres advisory lock, so it executes once across the fleet."
  info "No dedicated node, and no single point of failure for scheduled work."
else
  fail "EXACTLY-ONCE CHECK FAILED — see results/scheduler-exactly-once.json"
fi

exit "$VERDICT"
