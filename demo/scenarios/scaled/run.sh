#!/bin/bash
# SCALED SCENARIO — ONE-COMMAND, FULL-LIFECYCLE PROOF
#
# Brings the multi-replica stack up from any state, waits until every container
# is healthy, mints a token the scaled stack actually accepts, runs the fast
# proofs, captures all logs + JSON artifacts, prints one verdict, and (by
# default) leaves the stack up for inspection.
#
#   just scaled-demo                 # 3 replicas, leave stack up after
#   REPLICAS=5 just scaled-demo      # scale wider
#   KEEP=0 just scaled-demo          # tear the stack down at the end
#   SOAK=1 just scaled-demo          # also run the ~1h soak (02) — long!
#
# Why a wrapper: the individual 01..05 scripts assume a healthy stack already
# exists. Against a half-up stack (e.g. after a docker/WSL restart) they cascade
# into failures. This script owns the whole lifecycle so a single command is
# reproducible from a clean checkout or a broken stack alike.
#
# Cost: Free (no AI inference scenarios are run).

set -uo pipefail   # NOT -e: we want every proof to run and report, not abort early.

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
SCALED_DIR="$DEMO_ROOT/scenarios/scaled"
RESULTS_DIR="$SCALED_DIR/results"
LOG_DIR="$RESULTS_DIR/logs"
RUN_TS="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_LOG="$RESULTS_DIR/run-$RUN_TS.log"

REPLICAS="${REPLICAS:-3}"
KEEP="${KEEP:-1}"            # 1 = leave stack up (default); 0 = tear down at end
SOAK="${SOAK:-0}"           # 1 = also run the long soak (02)
HEALTH_TIMEOUT="${HEALTH_TIMEOUT:-300}"   # seconds to wait for all-healthy

DC=(docker compose -f "$COMPOSE_FILE")

mkdir -p "$LOG_DIR"

# All console output also lands in the run log.
exec > >(tee -a "$RUN_LOG") 2>&1

JWT_RE='eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+'

header "SCALED SCENARIO — FULL LIFECYCLE" "reset → up → health → proofs → verdict"
info "compose:  $COMPOSE_FILE"
info "replicas: $REPLICAS    keep-up: $KEEP    soak: $SOAK"
info "run log:  ${RUN_LOG#$PROJECT_DIR/}"

# ── Preflight: required tooling ────────────────
for tool in docker jq curl cargo; do
  command -v "$tool" >/dev/null 2>&1 || { fail "$tool is required."; exit 1; }
done
[[ -f "$COMPOSE_FILE" ]] || { fail "Scaled compose file not found: $COMPOSE_FILE"; exit 1; }

# ── Step 1: reset + build + up ─────────────────
divider
step "1/5  Reset any stale stack, then build and start $REPLICAS replicas"

# The scaled image COPIes host-built release binaries (the Dockerfile cannot see
# the sibling core repo, so it assembles a runtime layer around binaries built
# on the host where the path-patch resolves). Ensure they exist before building.
if [[ ! -x "$PROJECT_DIR/target/release/systemprompt" \
   || ! -x "$PROJECT_DIR/target/release/systemprompt-mcp-agent" ]]; then
  info "Release binaries missing — building them first (just build --release)"
  ( cd "$PROJECT_DIR" && just build --release ) || { fail "host release build failed"; exit 1; }
fi
# Stage binaries into a real dir inside the build context: `target` is a symlink
# to a shared cargo cache that buildkit cannot follow out of the context.
STAGE_BIN="$PROJECT_DIR/deploy/scenarios/scaled/.bin"
mkdir -p "$STAGE_BIN"
cp -L "$PROJECT_DIR/target/release/systemprompt"           "$STAGE_BIN/systemprompt"
cp -L "$PROJECT_DIR/target/release/systemprompt-mcp-agent" "$STAGE_BIN/systemprompt-mcp-agent"

"${DC[@]}" down -v --remove-orphans 2>/dev/null || true
if ! "${DC[@]}" up -d --build --scale app="$REPLICAS"; then
  fail "docker compose up failed — see output above."
  exit 1
fi
pass "stack started"

# ── Step 2: wait until every container is healthy ──
divider
step "2/5  Waiting up to ${HEALTH_TIMEOUT}s for all containers to report healthy"

# Services that MUST be healthy before we test. The read replica is wired for
# topology realism only (no read routing yet); we report it but do not block on it.
REQUIRED_SVCS=(postgres-primary app scheduler lb)

health_snapshot() {
  # One line per container: "<service>\t<state>\t<health>". Robust across compose
  # JSON shapes (array vs NDJSON) via `jq -s` + flatten.
  "${DC[@]}" ps --format json 2>/dev/null \
    | jq -rs 'flatten | .[] | "\(.Service)\t\(.State)\t\(.Health // "none")"' 2>/dev/null
}

# A snapshot is "ready" when every required service has >=1 container and all of
# its containers are running+healthy. One awk pass, no GNU-only grep flags.
snapshot_ready() {
  local snap="$1" required="$2"
  awk -F'\t' -v req="$required" '
    BEGIN { n = split(req, want, " ") }
    { state[$1]; seen[$1]++; if ($2 == "running" && $3 == "healthy") ok[$1]++ }
    END {
      for (i = 1; i <= n; i++) {
        s = want[i]
        if (!(s in seen)) exit 1            # service missing entirely
        if (ok[s] != seen[s]) exit 1        # some container not running+healthy
      }
      exit 0
    }' <<< "$snap"
}

deadline=$(( $(date +%s) + HEALTH_TIMEOUT ))
all_healthy=0
while (( $(date +%s) < deadline )); do
  snap="$(health_snapshot)"
  if [[ -n "$snap" ]] && snapshot_ready "$snap" "${REQUIRED_SVCS[*]}"; then
    all_healthy=1; break
  fi
  sleep 5
done

echo ""
info "container status:"
health_snapshot | sed 's/^/    /'
echo ""

if (( all_healthy != 1 )); then
  fail "Stack did not reach all-healthy within ${HEALTH_TIMEOUT}s."
  warn "Dumping logs of non-healthy containers, then aborting."
  while IFS=$'\t' read -r s state hlth; do
    if [[ "$state" != "running" || "$hlth" != "healthy" ]]; then
      echo "----- $s ($state/$hlth) -----"
      "${DC[@]}" logs --no-color --tail 40 "$s" 2>/dev/null | sed 's/^/    /'
    fi
  done <<< "$(health_snapshot)"
  exit 1
fi
pass "all required containers healthy"

# ── Step 3: mint a token the SCALED stack accepts ──
divider
step "3/5  Minting an admin token signed by the scaled stack's own key"
# The scaled stack has its OWN RSA signing key (shared across replicas) and its
# admin user lives in the scaled primary DB, so a token from the host `local`
# profile is rejected (401). Mint inside an app replica via `admin session
# login` — the same approach the airgap scenario uses. The admin user is seeded;
# create+promote idempotently in case a custom email is supplied.
ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-admin@localhost.dev}"
APP_CID="$("${DC[@]}" ps -q app | head -1)"
app_cli() { docker exec "$APP_CID" sh -c "systemprompt $* 2>&1"; }

app_cli "admin users create --name scaled-admin --email '$ADMIN_EMAIL'" >/dev/null 2>&1 || true
SC_UID="$(app_cli "admin users search '$ADMIN_EMAIL'" 2>/dev/null | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1)"
[[ -n "$SC_UID" ]] && app_cli "admin users role promote $SC_UID" >/dev/null 2>&1 || true

SCALED_TOKEN="$(app_cli "admin session login --email '$ADMIN_EMAIL' --token-only" | grep -oE "$JWT_RE" | head -1)"

if [[ -z "$SCALED_TOKEN" ]]; then
  warn "Could not mint a scaled token inside the app container ($ADMIN_EMAIL)."
  warn "The cross-replica event proof (03 part b) and quick-proof (05) need it;"
  warn "they will report a setup failure but the other proofs still run."
else
  export TOKEN="$SCALED_TOKEN"     # picked up by load_token in child scripts
  pass "scaled admin token minted (${#SCALED_TOKEN} chars, $ADMIN_EMAIL)"
fi

# ── Step 4: run the fast proofs ────────────────
divider
step "4/5  Running scaled proofs (capturing each to the run log)"

chmod +x "$SCALED_DIR"/0*.sh 2>/dev/null || true

RC_LOAD=0 RC_SOAK="-" RC_DIST=0 RC_SCHED=0 RC_QUICK=0

subheader "01 — load through the LB (p95 + error SLO)"
"$SCALED_DIR/01-load.sh"; RC_LOAD=$?

if [[ "$SOAK" == "1" ]]; then
  subheader "02 — sustained soak (long-running)"
  "$SCALED_DIR/02-soak.sh"; RC_SOAK=$?
fi

subheader "03 — replica distribution + real cross-replica event"
"$SCALED_DIR/03-replica-distribution.sh"; RC_DIST=$?

subheader "04 — scheduler isolation (no duplicate cron)"
"$SCALED_DIR/04-scheduler-isolation.sh"; RC_SCHED=$?

subheader "05 — throughput + audit-spine quick proof (through the LB)"
TARGET_URL="$LB_URL" "$SCALED_DIR/05-quick-proof.sh"; RC_QUICK=$?

# ── Capture per-container logs for the record ──
divider
step "Capturing per-container logs to results/logs/"
while IFS=$'\t' read -r s _ _; do
  [[ -z "$s" ]] && continue
  "${DC[@]}" logs --no-color "$s" > "$LOG_DIR/$s.log" 2>&1 || true
done <<< "$(health_snapshot | sort -u -k1,1)"
pass "logs written to ${LOG_DIR#$PROJECT_DIR/}"

# ── Step 5: single verdict ─────────────────────
divider
step "5/5  Verdict"

verdict_line() {  # name  rc   (rc "-" = skipped)
  local name="$1" rc="$2"
  if [[ "$rc" == "-" ]]; then printf "  %-34s ${YELLOW}skipped${R}\n" "$name"
  elif [[ "$rc" == "0" ]]; then printf "  %-34s ${GREEN}PASS${R}\n" "$name"
  else printf "  %-34s ${RED}FAIL (exit $rc)${R}\n" "$name"; fi
}

echo ""
verdict_line "01 load SLO"                "$RC_LOAD"
verdict_line "02 soak"                    "$RC_SOAK"
verdict_line "03 distribution + event"    "$RC_DIST"
verdict_line "04 scheduler isolation"     "$RC_SCHED"
verdict_line "05 throughput + audit spine" "$RC_QUICK"
echo ""

# Pull a few headline numbers straight from the JSON artifacts when present.
if [[ -f "$RESULTS_DIR/load.json" ]]; then
  p95=$(jq -r '[.scenarios[].p95_ms] | max' "$RESULTS_DIR/load.json" 2>/dev/null)
  err=$(jq -r '[.scenarios[].error_rate] | max' "$RESULTS_DIR/load.json" 2>/dev/null)
  info "load:  worst p95 ${p95}ms · worst error rate ${err}"
fi
if [[ -f "$RESULTS_DIR/replica-distribution.json" ]]; then
  ev=$(jq -r '.event_bus.status' "$RESULTS_DIR/replica-distribution.json" 2>/dev/null)
  sp=$(jq -r '.lb_spread.even_within_tolerance' "$RESULTS_DIR/replica-distribution.json" 2>/dev/null)
  info "dist:  lb_even=$sp · cross_replica_event=$ev"
fi

OVERALL=0
for rc in "$RC_LOAD" "$RC_DIST" "$RC_SCHED" "$RC_QUICK"; do
  [[ "$rc" == "0" ]] || OVERALL=1
done
[[ "$SOAK" == "1" && "$RC_SOAK" != "0" ]] && OVERALL=1

echo ""
if (( OVERALL == 0 )); then
  pass "SCALED SCENARIO PASSED — all proofs green."
else
  fail "SCALED SCENARIO FAILED — see the per-proof output above and results/."
fi

# ── Teardown policy ────────────────────────────
divider
if [[ "$KEEP" == "0" ]]; then
  step "Tearing down the stack (KEEP=0)"
  "${DC[@]}" down -v --remove-orphans 2>/dev/null || true
  pass "stack and volumes removed"
else
  info "Stack left running. Inspect it with:"
  echo "    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml ps"
  echo "    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml logs -f app"
  echo "    curl -s $LB_URL/api/v1/health"
  info "Tear it down when done:  just scaled-down   (or KEEP=0 just scaled-demo)"
fi

echo ""
info "Artifacts: ${RESULTS_DIR#$PROJECT_DIR/}/  (run log, logs/, *.json)"
exit "$OVERALL"
