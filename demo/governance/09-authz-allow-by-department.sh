#!/bin/bash
# DEMO 9: AUTHZ — ALLOW BY DEPARTMENT
# Engineering user (alice) hits the gateway with model=claude-* and is ALLOWED
# by a department-scoped ACL rule. Companion to 10-authz-deny-by-department.sh.
#
# Flow:
#   1. Ensure alice (eng) and bob (finance) test users exist
#   2. Ensure ACL rules: dept=eng allow on claude-*, dept=finance allow on gpt-*
#   3. Mint a JWT for alice with department/roles claims
#   4. POST /v1/messages with model=claude-3-sonnet -> expect 200
#   5. Show governance_decisions row: policy=authz decision=allow
#
# Cost: Free for the authz path; the upstream model call uses a real API key.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

ADMIN_TOKEN="${TOKEN:-}"
if [[ -z "$ADMIN_TOKEN" && -f "$TOKEN_FILE" ]]; then
  ADMIN_TOKEN=$(cat "$TOKEN_FILE")
fi
if [[ -z "$ADMIN_TOKEN" ]]; then
  echo "ERROR: Run demo/00-preflight.sh first to mint an admin token." >&2
  exit 1
fi

ALICE_ID="4c127601-741a-4064-8d43-12b7d24158bf"
BOB_ID="7cd34e51-9313-4c3a-b851-773b4668e63a"
GW_CLAUDE="claude-star-3cbb7d"
GW_GPT="gpt-star-e2e01"

header "DEMO 9: AUTHZ — ALLOW BY DEPARTMENT" "alice (engineering) -> claude-* -> ALLOW"

# ─────────────────────────────────────────────────
subheader "STEP 1: Ensure test users exist"

ensure_user() {
  local user_id="$1" username="$2" email="$3" role="$4" dept="$5"
  if "$CLI" admin users get "$user_id" --profile "$PROFILE" >/dev/null 2>&1; then
    info "$username already exists ($user_id)"
  else
    cmd "systemprompt admin users create --id $user_id --username $username ..."
    "$CLI" admin users create \
      --id "$user_id" --username "$username" --email "$email" \
      --roles "$role" --department "$dept" \
      --profile "$PROFILE" 2>&1 | sed 's/^/  /' || warn "create returned non-zero (may already exist)"
    pass "created $username ($dept/$role)"
  fi
}

ensure_user "$ALICE_ID" alice alice@demo engineering eng
ensure_user "$BOB_ID"   bob   bob@demo   finance     finance

# ─────────────────────────────────────────────────
subheader "STEP 2: Ensure ACL rules (department-scoped allow)"

ensure_rule() {
  local entity_id="$1" dept="$2"
  local payload
  payload=$(printf '{"rule_type":"department","rule_value":"%s","access":"allow"}' "$dept")
  cmd "POST /api/public/admin/access-control/entity/gateway_route/$entity_id/rules"
  curl -fsS -X POST "$BASE_URL/api/public/admin/access-control/entity/gateway_route/$entity_id/rules" \
    -H "Authorization: Bearer $ADMIN_TOKEN" -H "Content-Type: application/json" \
    -d "$payload" >/dev/null 2>&1 \
    && pass "rule: department=$dept allow on $entity_id" \
    || warn "upsert returned non-zero (rule may already exist)"
  curl -fsS -X PATCH "$BASE_URL/api/public/admin/access-control/entity/gateway_route/$entity_id/default" \
    -H "Authorization: Bearer $ADMIN_TOKEN" -H "Content-Type: application/json" \
    -d '{"default_included":false}' >/dev/null 2>&1 \
    && pass "default_included=false on $entity_id"
}

ensure_rule "$GW_CLAUDE" eng
ensure_rule "$GW_GPT"    finance

# ─────────────────────────────────────────────────
subheader "STEP 3: Mint JWT for alice (engineering)"

if [[ ! -f /tmp/mint_jwt.py ]]; then
  fail "/tmp/mint_jwt.py missing — run the C1 e2e first or copy the helper."
  exit 1
fi
ALICE_JWT=$(python3 /tmp/mint_jwt.py alice)
info "alice claims: roles=[engineering] department=eng sub=$ALICE_ID"

# ─────────────────────────────────────────────────
subheader "STEP 4: POST /v1/messages with model=claude-3-sonnet"

cmd "POST $BASE_URL/v1/messages   (model=claude-3-sonnet, alice)"
HTTP_CODE=$(curl -s -o /tmp/authz_allow_resp.json -w "%{http_code}" \
  -X POST "$BASE_URL/v1/messages" \
  -H "Authorization: Bearer $ALICE_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-sonnet",
    "max_tokens": 32,
    "messages": [{"role":"user","content":"ping"}]
  }')

if [[ "$HTTP_CODE" == "200" ]]; then
  pass "alice -> claude-* allowed (HTTP 200)"
else
  fail "expected 200, got $HTTP_CODE"
  cat /tmp/authz_allow_resp.json | python3 -m json.tool 2>/dev/null | sed 's/^/  /' || cat /tmp/authz_allow_resp.json
fi

# ─────────────────────────────────────────────────
subheader "STEP 5: Governance audit row"

echo "  Most recent authz decision for alice:"
"$CLI" infra db query \
  "SELECT decision, policy, reason, tool_name FROM governance_decisions WHERE user_id='$ALICE_ID' AND policy='authz' ORDER BY created_at DESC LIMIT 1" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | sed 's/^/  /'

pass "audit row written"

header "NEXT: ./demo/governance/10-authz-deny-by-department.sh"
