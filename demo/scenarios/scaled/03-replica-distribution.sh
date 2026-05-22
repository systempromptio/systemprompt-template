#!/bin/bash
# SCALED DEMO 3: REPLICA DISTRIBUTION + CROSS-REPLICA EVENT FAN-OUT
#
# Two assertions, both about how the scaled stack behaves with N>1 replicas.
#
#  (a) LOAD-BALANCER SPREAD — nginx round-robins requests across replicas.
#      The loadtest runner's `lb-fairness` scenario drives the LB and buckets
#      every response by the `x-served-by` header each replica stamps. We
#      assert no single replica took an unfair share (within tolerance) — an
#      empirical measurement, not a model of nginx's documented behaviour.
#
#  (b) CROSS-REPLICA EVENT FAN-OUT — THE EVENT/SSE BUS SCALES ACROSS REPLICAS.
#      The event/SSE bus is relayed cross-replica by `PostgresEventBridge`, a
#      LISTEN/NOTIFY relay started unconditionally at server boot. So an event
#      published on replica A is fanned out to subscribers on EVERY replica. A
#      subscriber pinned to replica B will RECEIVE it.
#      We open an SSE stream against replica B, publish an event via replica A,
#      and assert the replica-B subscriber received the event. That delivery is
#      the empirical proof that the bus scales across nodes and is therefore
#      the PASS condition for part (b).
#
# Per-replica addressing: scaled `app` replicas publish no host ports, so we
# discover container IPs with `docker inspect` and run per-replica curls from
# inside the docker network via `docker compose exec lb`.
#
# Cost: Free.

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

COMPOSE_FILE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
LB_URL="http://localhost:8088"
LOADTEST_MANIFEST="$PROJECT_DIR/../systemprompt-core/crates/tests/loadtest/Cargo.toml"
RESULTS_DIR="$DEMO_ROOT/scenarios/scaled/results"
OUT_FILE="$RESULTS_DIR/replica-distribution.json"
LB_FAIRNESS_FILE="$RESULTS_DIR/.lb-fairness.json"

# Spread tolerance: with R replicas a perfectly fair share is 1/R of requests.
# We allow any replica to be within +/- SPREAD_TOL of that fair share.
SPREAD_TOL=0.35   # 35 %
APP_PORT=8080     # internal container port (frozen contract)

DC=(docker compose -f "$COMPOSE_FILE")

header "SCALED DEMO 3: REPLICA DISTRIBUTION" "LB spread + cross-replica event fan-out"

# ── Preflight ──────────────────────────────────
step "Preflight checks"

for tool in jq docker curl cargo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    fail "$tool is required for this demo."
    exit 1
  fi
done
pass "jq, docker, curl, cargo present"

if [[ ! -f "$LOADTEST_MANIFEST" ]]; then
  fail "Load-test runner manifest not found: $LOADTEST_MANIFEST"
  info "Bucket 3 (../systemprompt-core) must be checked out alongside this repo."
  exit 1
fi
pass "load-test runner found"

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

# ── Discover replica container IPs ─────────────
divider
step "Discovering app replica containers"

mapfile -t REPLICA_IDS < <("${DC[@]}" ps -q app 2>/dev/null)
REPLICA_COUNT=${#REPLICA_IDS[@]}

if (( REPLICA_COUNT < 2 )); then
  fail "Need at least 2 app replicas for a distribution test; found $REPLICA_COUNT."
  info "Restart scaled with more replicas: just scaled-up REPLICAS=3"
  exit 1
fi
pass "found $REPLICA_COUNT app replicas"

declare -a REPLICA_IPS=()
for cid in "${REPLICA_IDS[@]}"; do
  ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$cid" 2>/dev/null)
  if [[ -z "$ip" ]]; then
    fail "Could not resolve container IP for $cid"
    exit 1
  fi
  REPLICA_IPS+=("$ip")
  info "replica ${cid:0:12} -> $ip"
done

# ── PART (a): load-balancer spread ─────────────
divider
subheader "PART (a): nginx round-robin spread across replicas"

# Empirical attribution, no modelling: the loadtest runner's `lb-fairness`
# scenario drives the LB and buckets every response by the `x-served-by`
# header each replica stamps (its instance_id = container hostname). The
# runner emits a `served_by: {instance: count}` map we assert directly on.
step "Running the lb-fairness scenario through the LB"
cmd "cargo run --manifest-path <loadtest>/Cargo.toml -- \\
      --profile default --scenario lb-fairness \\
      --base-url $LB_URL --output json --out-file results/.lb-fairness.json"

if ! cargo run --quiet --release --manifest-path "$LOADTEST_MANIFEST" -- \
      --profile default \
      --scenario lb-fairness \
      --token "${SYSTEMPROMPT_TOKEN:-anon}" \
      --base-url "$LB_URL" \
      --output json \
      --out-file "$LB_FAIRNESS_FILE" >/dev/null 2>&1; then
  warn "lb-fairness runner exited non-zero — reading its report anyway."
fi

if [[ ! -f "$LB_FAIRNESS_FILE" ]]; then
  fail "No lb-fairness report produced at $LB_FAIRNESS_FILE"
  exit 1
fi

SERVED_BY=$(jq -c '.scenarios["lb-fairness"].served_by // {}' "$LB_FAIRNESS_FILE")
if [[ "$SERVED_BY" == "{}" ]]; then
  fail "lb-fairness report has an empty served_by map — no requests attributed."
  exit 1
fi

# An "unknown" bucket means responses arrived without an x-served-by header.
# A tiny fraction (<0.5%) is tolerated — connection races during nginx upstream
# selection / replica boot can produce rare responses without the header before
# the middleware chain is fully warm. A large fraction means x-served-by is
# missing in steady state and the fairness numbers cannot be trusted.
UNKNOWN=$(jq -r '."unknown" // 0' <<<"$SERVED_BY")
TOTAL_TMP=$(jq -r '[.[]] | add' <<<"$SERVED_BY")
UNKNOWN_PCT=$(jq -n --argjson u "$UNKNOWN" --argjson t "$TOTAL_TMP" \
  'if $t == 0 then 0 else ($u/$t*100*100|round)/100 end')
UNKNOWN_OK=1
if [[ "$UNKNOWN" -gt 0 ]]; then
  if jq -e -n --argjson p "$UNKNOWN_PCT" '$p <= 0.5' >/dev/null; then
    warn "$UNKNOWN/$TOTAL_TMP requests bucketed 'unknown' (${UNKNOWN_PCT}%) — within 0.5% tolerance"
  else
    fail "$UNKNOWN/$TOTAL_TMP requests bucketed 'unknown' (${UNKNOWN_PCT}%) — exceeds 0.5%"
    warn "The app may not be stamping x-served-by on /health responses in steady state."
    UNKNOWN_OK=0
  fi
fi

TOTAL_REQ=$(jq -r '[.[]] | add' <<<"$SERVED_BY")
INSTANCE_COUNT=$(jq -r 'keys | map(select(. != "unknown")) | length' <<<"$SERVED_BY")
FAIR=$(jq -n --argjson t "$TOTAL_REQ" --argjson r "$REPLICA_COUNT" 'if $r == 0 then 0 else $t / $r end')
LOW=$(jq -n --argjson f "$FAIR" --argjson t "$SPREAD_TOL" '$f * (1 - $t)')
HIGH=$(jq -n --argjson f "$FAIR" --argjson t "$SPREAD_TOL" '$f * (1 + $t)')
info "total attributed requests: $TOTAL_REQ across $INSTANCE_COUNT instance(s)"
info "fair share per replica: ~$(jq -n --argjson f "$FAIR" '($f*10|round)/10')  (band $(jq -n --argjson l "$LOW" '($l*10|round)/10')..$(jq -n --argjson h "$HIGH" '($h*10|round)/10'))"

SPREAD_OK=1
(( UNKNOWN_OK == 1 )) || SPREAD_OK=0

if (( INSTANCE_COUNT != REPLICA_COUNT )); then
  fail "lb-fairness saw $INSTANCE_COUNT distinct instances; expected $REPLICA_COUNT replicas."
  SPREAD_OK=0
fi

# Per-instance band check. SPREAD_JSON keeps the {instance,hits} breakdown.
SPREAD_JSON="[]"
while IFS=$'\t' read -r inst hits; do
  [[ -z "$inst" ]] && continue
  SPREAD_JSON=$(jq -n --argjson arr "$SPREAD_JSON" --arg i "$inst" --argjson h "$hits" \
    '$arr + [{instance: $i, hits: $h}]')
  if [[ "$inst" == "unknown" ]]; then
    # Already handled above against the 0.5% tolerance.
    continue
  fi
  if jq -e -n --argjson h "$hits" --argjson l "$LOW" --argjson hi "$HIGH" \
       '$h >= $l and $h <= $hi' >/dev/null; then
    pass "instance $inst served $hits/$TOTAL_REQ requests (within band)"
  else
    fail "instance $inst served $hits/$TOTAL_REQ requests (outside band)"
    SPREAD_OK=0
  fi
done < <(jq -r 'to_entries[] | "\(.key)\t\(.value)"' <<<"$SERVED_BY")

# ── PART (b): cross-replica event-bus fan-out ──
divider
subheader "PART (b): event bus — cross-replica delivery via PostgresEventBridge"
info "Subscribe on replica B, route a real A2A event on replica A as the same"
info "user, and assert replica B receives it. Delivery across two processes is"
info "the empirical proof that PostgresEventBridge relays the bus across replicas."

# The demo user's token authenticates the SSE subscription, the context create,
# and the event route. The subscriber and publisher MUST be the same user — the
# A2A broadcaster is user-scoped, so the relayed event is only delivered to that
# user's connections. Requires demo/.token (run 00-preflight.sh first).
load_token

# Run all curls from inside an app replica: it has curl (its healthcheck uses it)
# and sits on the scaled network so it can reach every replica by container IP.
# nginx:alpine ships wget, not curl, so the lb container cannot drive these.
EXEC_CID="${REPLICA_IDS[0]}"
REPLICA_A="${REPLICA_IPS[0]}"   # publisher: routes the event
REPLICA_B="${REPLICA_IPS[1]}"   # subscriber: must receive it cross-process
info "exec host  -> app replica ${EXEC_CID:0:12}"
info "publisher  -> replica A: $REPLICA_A"
info "subscriber -> replica B: $REPLICA_B"

# Real core surfaces (core 0.11):
#   POST /api/v1/core/contexts/            -> create a context owned by the user
#   GET  /api/v1/stream/a2a                -> SSE stream of the user's A2A events
#   POST /api/v1/core/contexts/{id}/events -> forward_event -> EventRouter::route_a2a
# route_a2a writes an event_outbox row + Postgres NOTIFY; the PostgresEventBridge
# on every replica consumes it and re-injects into the local A2A_BROADCASTER.
CONTEXTS_PATH="/api/v1/core/contexts"
A2A_STREAM_PATH="/api/v1/stream/a2a"
EVENT_TOKEN="scaled-evt-$(date +%s)-$RANDOM"
LISTEN_SECS=12

# Defaults so the artifact + verdict are well-defined even if setup fails early.
CROSS_DELIVERED=false
EVENT_OK=0
EVENT_STATUS="not run"
PUBLISH_STATUS="000"
CONTEXT_ID=""

# forward_event refuses to route unless the caller owns the target context, so
# create one first (via replica A, as the demo user).
step "Creating a context on replica A (owned by the demo user)"
# No trailing slash on the collection POST: the trailing slash 308-redirects.
CREATE_RESP=$(docker exec "$EXEC_CID" sh -c \
  "curl -sSL --max-time 8 -X POST \
     -H 'Authorization: Bearer $TOKEN' \
     -H 'Content-Type: application/json' \
     -d '{\"name\":\"scaled-event-proof\"}' \
     http://${REPLICA_A}:${APP_PORT}${CONTEXTS_PATH}" 2>/dev/null || true)
CONTEXT_ID=$(echo "$CREATE_RESP" | jq -r '.data.context_id // empty' 2>/dev/null)

if [[ -z "$CONTEXT_ID" ]]; then
  fail "Could not create a context for the event proof — cannot prove cross-replica delivery."
  info "create response: ${CREATE_RESP:0:300}"
  EVENT_STATUS="setup failed (context create)"
else
  pass "context created: $CONTEXT_ID"

  step "Opening A2A SSE stream on replica B for ${LISTEN_SECS}s"
  SSE_CAPTURE=$(mktemp)
  docker exec "$EXEC_CID" sh -c \
    "timeout ${LISTEN_SECS} curl -fsS -N --max-time ${LISTEN_SECS} \
       -H 'Authorization: Bearer $TOKEN' \
       -H 'Accept: text/event-stream' \
       http://${REPLICA_B}:${APP_PORT}${A2A_STREAM_PATH}" \
    > "$SSE_CAPTURE" 2>/dev/null &
  SSE_PID=$!

  # Let the subscriber fully establish (and its bridge LISTEN settle) before routing.
  sleep 3

  step "Routing an A2A event on replica A (token: $EVENT_TOKEN)"
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  TASK_ID=$(uuidgen 2>/dev/null | tr 'A-Z' 'a-z' || cat /proc/sys/kernel/random/uuid 2>/dev/null)
  EVENT_BODY="{\"protocol\":\"a2a\",\"event\":{\"type\":\"TASK_STATUS_UPDATE\",\"timestamp\":\"${TS}\",\"taskId\":\"${TASK_ID}\",\"contextId\":\"${CONTEXT_ID}\",\"state\":\"TASK_STATE_WORKING\",\"message\":\"${EVENT_TOKEN}\"}}"
  PUBLISH_STATUS=$(docker exec "$EXEC_CID" sh -c \
    "curl -sS -o /dev/null -w '%{http_code}' --max-time 8 -X POST \
       -H 'Authorization: Bearer $TOKEN' \
       -H 'Content-Type: application/json' \
       -d '$EVENT_BODY' \
       http://${REPLICA_A}:${APP_PORT}${CONTEXTS_PATH}/${CONTEXT_ID}/events" 2>/dev/null || echo "000")
  info "forward_event HTTP status: $PUBLISH_STATUS"

  # Wait for the listen window to close, then check what replica B captured.
  wait "$SSE_PID" 2>/dev/null || true

  divider
  step "Checking whether replica B received the event routed on replica A"
  if grep -q "$EVENT_TOKEN" "$SSE_CAPTURE" 2>/dev/null; then
    CROSS_DELIVERED=true
    EVENT_STATUS="delivered"
    EVENT_OK=1
    pass "Replica B RECEIVED the A2A event routed on replica A."
    info "PostgresEventBridge relayed it via Postgres LISTEN/NOTIFY — the bus scales across replicas."
  elif [[ "$PUBLISH_STATUS" =~ ^2[0-9][0-9]$ ]]; then
    EVENT_STATUS="not delivered (route HTTP $PUBLISH_STATUS)"
    fail "Replica B received NOTHING despite forward_event returning $PUBLISH_STATUS."
    warn "PostgresEventBridge appears not to be relaying — investigate cross-replica fan-out."
  else
    EVENT_STATUS="route failed (HTTP $PUBLISH_STATUS)"
    fail "forward_event returned $PUBLISH_STATUS — the event was never routed."
    warn "Check the token audience/scope and context ownership. SSE capture (first 300B):"
    head -c 300 "$SSE_CAPTURE" 2>/dev/null | sed 's/^/    /'
  fi
  rm -f "$SSE_CAPTURE"
fi

# ── Persist artifact ───────────────────────────
jq -n \
  --argjson requests "$TOTAL_REQ" \
  --argjson replicas "$REPLICA_COUNT" \
  --argjson spread "$SPREAD_JSON" \
  --argjson spread_ok "$([[ $SPREAD_OK -eq 1 ]] && echo true || echo false)" \
  --arg replica_a "$REPLICA_A" --arg replica_b "$REPLICA_B" \
  --argjson cross_delivered "$CROSS_DELIVERED" \
  --arg pub_status "$PUBLISH_STATUS" \
  --arg event_status "$EVENT_STATUS" \
  '{
     lb_spread: { method: "loadtest lb-fairness scenario (x-served-by attribution)", requests: $requests, replicas: $replicas, per_replica: $spread, even_within_tolerance: $spread_ok },
     event_bus: {
       publisher_replica: $replica_a,
       subscriber_replica: $replica_b,
       publish_http_status: $pub_status,
       cross_replica_event_delivered: $cross_delivered,
       expected_cross_replica_delivered: true,
       status: $event_status,
       method: "create context + GET /api/v1/stream/a2a on replica B + POST /api/v1/core/contexts/{id}/events on replica A (forward_event -> EventRouter::route_a2a)",
       note: "An A2A event routed on replica A is delivered to a subscriber on replica B via PostgresEventBridge (event_outbox + Postgres LISTEN/NOTIFY). The core integration test crates/tests/integration/events/cross_replica.rs is the authoritative regression for the relay itself."
     },
     passed: ($spread_ok and ($cross_delivered == true))
   }' > "$OUT_FILE"
pass "report written: results/replica-distribution.json"

# ── Verdict ────────────────────────────────────
divider
VERDICT=0
(( SPREAD_OK == 1 )) || { fail "LB spread outside tolerance"; VERDICT=1; }
(( EVENT_OK  == 1 )) || { fail "Cross-replica event-bus behaviour unexpected"; VERDICT=1; }

if (( VERDICT == 0 )); then
  pass "REPLICA DISTRIBUTION VERIFIED"
  info "LB spreads evenly; event bus fans out across replicas via PostgresEventBridge."
  info "Next: ./demo/scenarios/scaled/04-scheduler-isolation.sh"
else
  fail "REPLICA DISTRIBUTION CHECK FAILED — see results/replica-distribution.json"
fi

exit "$VERDICT"
