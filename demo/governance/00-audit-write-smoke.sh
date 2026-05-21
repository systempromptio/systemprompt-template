#!/bin/bash
# GOVERNANCE AUDIT-WRITE SMOKE
# Asserts that a single POST to /api/public/hooks/govern lands at least one
# row in governance_decisions for the synthesised session id. Runs first
# (alphabetic order) in demo/governance/ so sweep catches a silent INSERT
# drop the moment it appears in CI, instead of three days later.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"
load_token

SID="audit-smoke-$(date +%s)-$$"

curl -fsS -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\",\"agent_id\":\"developer_agent\",\"session_id\":\"$SID\",\"cwd\":\"/tmp\",\"tool_input\":{\"file_path\":\"/x\"}}" \
  > /dev/null

# Audit write is fire-and-forget via tokio::spawn — give it a beat to land.
sleep 1

count=$("$CLI" --profile "$PROFILE" infra db query \
  "SELECT COUNT(*)::int AS c FROM governance_decisions WHERE session_id = '$SID'" \
  2>/dev/null | sed -n 's/.*"c"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)

if [[ "${count:-0}" -lt 1 ]]; then
    fail "governance audit write smoke: expected >=1 row for session=$SID, got ${count:-0}"
    exit 1
fi

pass "governance audit write smoke: ${count} row(s) written for session=$SID"
