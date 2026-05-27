#!/bin/bash
# DEMO: RATE LIMITING — POLICY CONFIG + LIVE ENFORCEMENT
# Shows the gateway rate-limit posture (HTTP-layer config) and then
# proves the governance-layer `rate_limit` policy actually denies once
# a single (session_id, user_id) caller blows past its sliding window.
#
# What this does:
#   1. Prints the HTTP rate-limit config (admin config rate-limits show)
#   2. Prints the governance rate_limit policy config from
#      services/governance/config.yaml (requests_per_window / window_secs)
#   3. Fires N+20 PreToolUse calls against /api/public/hooks/govern with
#      a single freshly-generated session_id, where N = the policy limit
#   4. Counts allow vs deny in the responses
#   5. Queries governance_decisions to confirm the denies were attributed
#      to policy=rate_limit
#
# Cost: Free

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"
load_token

header "GOVERNANCE: RATE LIMITING" "HTTP config + live policy enforcement"

subheader "STEP 1: HTTP-layer rate-limit config"
run_cli_indented admin config rate-limits show

subheader "STEP 2: Governance rate_limit policy"
echo "  $ cat services/governance/config.yaml  # rate_limit policy"
echo ""
awk '/- id: rate_limit/,/^  - id:/' "$PROJECT_DIR/services/governance/config.yaml" \
  | grep -v '^  - id: [^r]' | sed 's/^/    /'
echo ""

# Read the configured limit so the demo stays in sync with the YAML.
RL_LIMIT=$(awk '/- id: rate_limit/{f=1} f && /requests_per_window:/{print $2; exit}' \
  "$PROJECT_DIR/services/governance/config.yaml")
RL_LIMIT=${RL_LIMIT:-300}
OVERSHOOT=20
TOTAL=$((RL_LIMIT + OVERSHOOT))

subheader "STEP 3: Hammer /api/public/hooks/govern with $TOTAL calls"
SID="demo-ratelimit-$(date +%s)-$$"
info "session_id = $SID  (one caller, one sliding window)"
info "policy limit = $RL_LIMIT requests / 60s  →  expect ~$OVERSHOOT denies"
echo ""

ALLOWED=0
DENIED=0
for i in $(seq 1 "$TOTAL"); do
  RESP=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"/tmp/x\"},\"agent_id\":\"developer_agent\",\"session_id\":\"$SID\",\"cwd\":\"/tmp\"}")
  if echo "$RESP" | grep -q '"deny"'; then
    DENIED=$((DENIED + 1))
  else
    ALLOWED=$((ALLOWED + 1))
  fi
done

echo "  allowed: $ALLOWED"
echo "  denied:  $DENIED"
echo ""
if [[ "$DENIED" -ge 1 ]]; then
  pass "Rate-limit policy enforced — $DENIED of $TOTAL calls denied past the window"
else
  fail "Expected at least one deny once limit exceeded; got $DENIED"
  exit 1
fi

subheader "STEP 4: Audit trail — decisions for this session"
"$CLI" infra db query \
  "SELECT decision, policy, COUNT(*) AS count FROM governance_decisions WHERE session_id = '$SID' GROUP BY decision, policy ORDER BY decision, policy" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | sed 's/^/  /'

echo ""
echo "  Sample deny rows:"
"$CLI" infra db query \
  "SELECT decision, policy, reason FROM governance_decisions WHERE session_id = '$SID' AND decision = 'deny' LIMIT 3" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | sed 's/^/  /'

header "RATE LIMITING DEMO COMPLETE"
