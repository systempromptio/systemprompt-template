#!/bin/bash
# SCALED DEMO 5: ~30-SECOND THROUGHPUT PROOF + AUDIT SPINE
#
# A short, executive-facing proof that replaces the 5-minute soak with a
# one-screen verdict in under a minute. Backs the headline numbers on the
# Scaled Deployment Factsheet:
#
#   • Throughput against the gateway (req/s)
#   • Fairness across replicas (max/min hit ratio via x-served-by) — only
#     meaningful when pointed at a multi-replica LB
#   • Audit-row count == requests sent (single audit spine)
#
# By default this hits the single-instance dev server at the BASE_URL derived
# from your active profile (typically http://localhost:8080). To prove
# multi-replica fairness, bring up the scaled stack and override TARGET_URL:
#
#   just scaled-up REPLICAS=3
#   TARGET_URL=http://localhost:8088 ./demo/scenarios/scaled/05-quick-proof.sh
#
# Cost: Free. Runtime target: < 45s.

set -e

source "$(cd "$(dirname "$0")/../.." && pwd)/_common.sh"

PROFILE="${1:-local}"
load_token

TARGET_URL="${TARGET_URL:-$BASE_URL}"
N="${N:-600}"
C="${C:-60}"
SESSION="quickproof-$(date +%s)"

# Classify target so the banner and verdict don't lie about what was tested.
# The scaled stack's nginx LB lives on :8088; anything else is "single
# instance" (a `just start` dev binary or an arbitrary host).
if [[ "$TARGET_URL" == *":8088"* ]]; then
  MODE="scaled"
  MODE_LABEL="nginx LB -> N app replicas ($TARGET_URL)"
else
  MODE="single"
  MODE_LABEL="single instance ($TARGET_URL) — fairness check will be trivial"
fi

# Audit-spine queries must hit the DB the target actually writes to. In scaled
# mode that is the scaled stack's own primary (inside the postgres-primary
# container), NOT the host `local` profile DB — querying the wrong DB is what
# made this proof report a false "audit delta 0" failure. In single mode, use
# the host CLI against the active profile.
SCALED_COMPOSE="$PROJECT_DIR/deploy/scenarios/scaled/docker-compose.scaled.yml"
db_scalar() {  # $1 = SQL returning a single scalar; echoes the raw value
  if [[ "$MODE" == "scaled" ]]; then
    docker compose -f "$SCALED_COMPOSE" exec -T postgres-primary \
      psql -U systemprompt -d systemprompt -tAc "$1" 2>/dev/null | tr -d '[:space:]'
  else
    "$CLI" infra db query "$1" --profile "$PROFILE" 2>/dev/null \
      | grep -oE '[0-9]+(\.[0-9]+)?' | head -1
  fi
}
db_rows() {  # $1 = SQL; echoes pipe-separated rows (scaled) or CLI JSON (single)
  if [[ "$MODE" == "scaled" ]]; then
    docker compose -f "$SCALED_COMPOSE" exec -T postgres-primary \
      psql -U systemprompt -d systemprompt -tA -F '|' -c "$1" 2>/dev/null
  else
    "$CLI" infra db query "$1" --profile "$PROFILE" 2>/dev/null
  fi
}

echo ""
echo "=========================================="
echo "  SCALED QUICK PROOF: ${N} reqs / ${C} concurrent"
echo "  Target: ${MODE_LABEL}"
echo "  Session: ${SESSION}"
echo "=========================================="
echo ""

if [[ "$MODE" == "single" ]]; then
  echo "  NOTE: not hitting the scaled stack. To prove multi-replica fairness:"
  echo "    just scaled-up REPLICAS=3"
  echo "    TARGET_URL=http://localhost:8088 $0"
  echo ""
fi

# ── Preflight: target must be reachable ────────
if ! curl -fsS -o /dev/null --max-time 5 "$TARGET_URL/api/v1/health"; then
  echo "  FAIL: target not reachable at $TARGET_URL"
  echo "  Single instance:  just start"
  echo "  Multi-replica:    just scaled-up REPLICAS=3 && TARGET_URL=http://localhost:8088 $0"
  exit 1
fi
echo "  Healthy at $TARGET_URL"
echo ""

# ── Baseline: prove the session is unique (audit row count starts at 0) ──
BASELINE_ROWS=$(db_scalar \
  "SELECT count(*) AS n FROM governance_decisions WHERE session_id = '$SESSION'")
BASELINE_ROWS="${BASELINE_ROWS:-0}"
echo "  Baseline audit rows for session: ${BASELINE_ROWS} (expected 0)"
echo ""

install_hey || exit 1

# jq is required for parsing CLI db-query JSON output.
if ! command -v jq >/dev/null 2>&1; then
  echo "  FAIL: jq not installed. brew install jq  /  apt-get install jq" >&2
  exit 1
fi

# ── Fire the load through the LB ───────────────
echo "------------------------------------------"
echo "  Firing ${N} governance decisions through the LB"
echo "------------------------------------------"
echo ""

PAYLOAD='{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$SESSION'","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/src/main.rs"}}'

HEY_OUT=$("$HEY" -n "$N" -c "$C" -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" \
  "$TARGET_URL/api/public/hooks/govern?plugin_id=enterprise-demo" 2>&1)

RPS=$(echo "$HEY_OUT" | grep "Requests/sec" | awk '{printf "%.0f", $2}')
P50=$(echo "$HEY_OUT" | grep "50% in" | awk '{printf "%.1f", $3 * 1000}')
P99=$(echo "$HEY_OUT" | grep "99% in" | awk '{printf "%.1f", $3 * 1000}')
OK_COUNT=$(echo "$HEY_OUT" | grep -E "\[200\]" | grep -oE '[0-9]+' | tail -1 || echo "0")

# ── Replica fairness via x-served-by ───────────
# Re-hit the LB with HEAD requests just to bucket x-served-by; cheap and
# attributable. 120 probes give a tight enough distribution for ±10% checks.
echo "------------------------------------------"
echo "  Sampling x-served-by across 120 probes"
echo "------------------------------------------"
echo ""

declare -A HITS
PROBES=120
for _ in $(seq 1 "$PROBES"); do
  SB=$(curl -sS -o /dev/null -D - --max-time 3 \
        -H "Authorization: Bearer $TOKEN" \
        "$TARGET_URL/api/v1/health" \
       | grep -i '^x-served-by:' | head -1 \
       | awk '{print $2}' | tr -d '\r')
  [[ -z "$SB" ]] && SB="unknown"
  HITS[$SB]=$(( ${HITS[$SB]:-0} + 1 ))
done

MIN=999999; MAX=0; REPLICA_COUNT=0
for inst in "${!HITS[@]}"; do
  v=${HITS[$inst]}
  [[ "$inst" == "unknown" ]] && continue
  REPLICA_COUNT=$(( REPLICA_COUNT + 1 ))
  (( v < MIN )) && MIN=$v
  (( v > MAX )) && MAX=$v
  printf "  replica %-32s %3d hits\n" "$inst" "$v"
done
echo ""

if (( REPLICA_COUNT == 0 )); then
  FAIRNESS="n/a"
  FAIR_OK=0
else
  FAIRNESS=$(awk -v mx="$MAX" -v mn="$MIN" 'BEGIN { if (mn==0) print "inf"; else printf "%.2f", mx/mn }')
  FAIR_OK=$(awk -v mx="$MAX" -v mn="$MIN" 'BEGIN { if (mn==0) print 0; else if (mx/mn <= 1.5) print 1; else print 0 }')
fi

# ── Audit-spine assertion ──────────────────────
# Pull row count, decision/policy distribution, and time range in a single
# query so we can show the user the audit spine actually moved — not just that
# a number changed.
echo "------------------------------------------"
echo "  Audit spine: querying Postgres for ${SESSION}"
echo "------------------------------------------"
echo ""

AUDIT_ROWS=$(db_scalar \
  "SELECT count(*) AS n FROM governance_decisions WHERE session_id = '$SESSION'")
AUDIT_ROWS="${AUDIT_ROWS:-0}"
AUDIT_DELTA=$(( AUDIT_ROWS - BASELINE_ROWS ))

# Server-side ingest span — derives true throughput independent of hey.
SPAN_S=$(db_scalar \
  "SELECT EXTRACT(EPOCH FROM (max(created_at) - min(created_at)))::float FROM governance_decisions WHERE session_id = '$SESSION'")
FIRST_SEEN=$(db_scalar "SELECT min(created_at) FROM governance_decisions WHERE session_id = '$SESSION'")
LAST_SEEN=$(db_scalar "SELECT max(created_at) FROM governance_decisions WHERE session_id = '$SESSION'")
FIRST_SEEN="${FIRST_SEEN:-?}"; LAST_SEEN="${LAST_SEEN:-?}"; SPAN_S="${SPAN_S:-0}"
SERVER_RPS="n/a"
if [[ -n "$SPAN_S" && "$SPAN_S" != "0" && "${AUDIT_ROWS:-0}" -gt 0 ]]; then
  SERVER_RPS=$(awk -v n="$AUDIT_ROWS" -v s="$SPAN_S" 'BEGIN { if (s>0) printf "%.0f", n/s; else print "n/a" }')
fi

# Decision / policy histogram — proves the four-stage pipeline actually ran.
echo "  Decision histogram:"
HIST=$(db_rows \
  "SELECT count(*) AS n, decision, policy FROM governance_decisions WHERE session_id = '$SESSION' GROUP BY decision, policy ORDER BY n DESC")
if [[ "$MODE" == "scaled" ]]; then
  echo "$HIST" | awk -F'|' 'NF>=3 { printf "    %s × decision=%s policy=%s\n", $1,$2,$3 }' || echo "    (no rows)"
  [[ -z "$HIST" ]] && echo "    (no rows)"
else
  echo "$HIST" | jq -r '.rows[] | "    \(.n) × decision=\(.decision) policy=\(.policy)"' 2>/dev/null || echo "    (no rows)"
fi
echo ""

# Sample three audit rows so the user can eyeball real data, not a count.
echo "  Sample audit rows (first 3 of ${AUDIT_ROWS}):"
SAMPLE=$(db_rows \
  "SELECT created_at, id, tool_name, decision, policy, plugin_id FROM governance_decisions WHERE session_id = '$SESSION' ORDER BY created_at LIMIT 3")
if [[ "$MODE" == "scaled" ]]; then
  echo "$SAMPLE" | awk -F'|' 'NF>=6 { printf "    %s  id=%s  tool=%s  decision=%s  policy=%s  plugin=%s\n", $1,$2,$3,$4,$5,$6 }' || echo "    (none)"
  [[ -z "$SAMPLE" ]] && echo "    (none)"
else
  echo "$SAMPLE" | jq -r '.rows[] | "    \(.created_at)  id=\(.id)  tool=\(.tool_name)  decision=\(.decision)  policy=\(.policy)  plugin=\(.plugin_id)"' 2>/dev/null || echo "    (none)"
fi
echo ""

# ── Verdict ────────────────────────────────────
echo "=========================================="
echo "  VERDICT"
echo "=========================================="
echo ""
printf "  Mode                         %s\n" "$MODE"
printf "  Client throughput (hey)      %s req/s\n" "${RPS:-?}"
printf "  Server-side throughput       %s req/s  (audit rows / DB time span)\n" "$SERVER_RPS"
printf "  Latency  p50 / p99           %sms / %sms\n" "${P50:-?}" "${P99:-?}"
printf "  Replicas hit (x-served-by)   %d\n" "$REPLICA_COUNT"
printf "  Fairness (max/min hits)      %s\n" "$FAIRNESS"
printf "  HTTP 200 / requests sent     %s / %s\n" "${OK_COUNT:-?}" "$N"
printf "  Audit rows: baseline -> now  %s -> %s   (delta %s, expected %s)\n" \
  "$BASELINE_ROWS" "$AUDIT_ROWS" "$AUDIT_DELTA" "$N"
printf "  Audit time range             %s -> %s  (%ss)\n" \
  "${FIRST_SEEN:-?}" "${LAST_SEEN:-?}" "${SPAN_S:-?}"
echo ""

VERDICT=0
[[ "${OK_COUNT:-0}" == "$N" ]] || { echo "  FAIL: not all requests returned 200"; VERDICT=1; }
[[ "${AUDIT_DELTA:-0}" == "$N" ]] || { echo "  FAIL: audit delta != requests sent (single-spine claim violated)"; VERDICT=1; }
if [[ "$MODE" == "scaled" ]]; then
  (( FAIR_OK == 1 )) || { echo "  FAIL: replica fairness ratio > 1.5"; VERDICT=1; }
  (( REPLICA_COUNT >= 2 )) || { echo "  FAIL: expected ≥2 replicas in scaled mode, saw ${REPLICA_COUNT}"; VERDICT=1; }
fi

if (( VERDICT == 0 )); then
  if [[ "$MODE" == "scaled" ]]; then
    echo "  PASS: LB spreads evenly · every request audited · single spine."
  else
    echo "  PASS (single-instance): every request audited · single spine."
    echo "        (fairness not exercised — re-run with scaled stack for that claim)"
  fi
fi

echo ""
echo "  See the rows yourself:"
if [[ "$MODE" == "scaled" ]]; then
  echo "    docker compose -f $SCALED_COMPOSE exec -T postgres-primary \\"
  echo "      psql -U systemprompt -d systemprompt -c \"SELECT * FROM governance_decisions WHERE session_id = '$SESSION' LIMIT 10\""
else
  echo "    $CLI infra db query \"SELECT * FROM governance_decisions WHERE session_id = '$SESSION' LIMIT 10\" --profile $PROFILE"
fi
echo "    $CLI infra logs trace list --limit 20"
echo ""

exit "$VERDICT"
