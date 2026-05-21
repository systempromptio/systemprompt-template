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

echo ""
echo "=========================================="
echo "  SCALED QUICK PROOF: ${N} reqs / ${C} concurrent"
echo "  LB ${LB_URL} -> N app replicas"
echo "=========================================="
echo ""

# ── Preflight: target must be reachable ────────
if ! curl -fsS -o /dev/null --max-time 5 "$TARGET_URL/api/v1/health"; then
  echo "  FAIL: target not reachable at $TARGET_URL"
  echo "  Single instance:  just start"
  echo "  Multi-replica:    just scaled-up REPLICAS=3 && TARGET_URL=http://localhost:8088 $0"
  exit 1
fi
echo "  Healthy at $TARGET_URL"
echo ""

install_hey || exit 1

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
echo "------------------------------------------"
echo "  Audit spine: counting rows for ${SESSION}"
echo "------------------------------------------"
echo ""

AUDIT_ROWS=$("$CLI" infra db query \
  "SELECT count(*) AS n FROM governance_decisions WHERE session_id = '$SESSION'" \
  --profile "$PROFILE" 2>/dev/null \
  | grep -oE '"n": [0-9]+' | head -1 | awk '{print $2}')
AUDIT_ROWS="${AUDIT_ROWS:-0}"

# ── Verdict ────────────────────────────────────
echo "=========================================="
echo "  VERDICT"
echo "=========================================="
echo ""
printf "  Throughput through LB        %s req/s\n" "${RPS:-?}"
printf "  Latency  p50 / p99           %sms / %sms\n" "${P50:-?}" "${P99:-?}"
printf "  Replicas hit (x-served-by)   %d\n" "$REPLICA_COUNT"
printf "  Fairness (max/min hits)      %s\n" "$FAIRNESS"
printf "  HTTP 200 / requests sent     %s / %s\n" "${OK_COUNT:-?}" "$N"
printf "  Audit rows for session       %s\n" "$AUDIT_ROWS"
echo ""

VERDICT=0
[[ "${OK_COUNT:-0}" == "$N" ]] || { echo "  FAIL: not all requests returned 200"; VERDICT=1; }
[[ "${AUDIT_ROWS:-0}" == "$N" ]] || { echo "  FAIL: audit row count != requests sent (single-spine claim violated)"; VERDICT=1; }
(( FAIR_OK == 1 )) || { echo "  FAIL: replica fairness ratio > 1.5"; VERDICT=1; }

if (( VERDICT == 0 )); then
  echo "  PASS: LB spreads evenly · every request audited · single spine."
fi
exit "$VERDICT"
