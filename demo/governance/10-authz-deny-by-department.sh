#!/bin/bash
# DEMO 10: AUTHZ — DENY BY DEPARTMENT
# Same fixtures as 09. alice (eng) posts to gpt-* and is DENIED. bob (finance)
# posts to gpt-* and is ALLOWED. Same model, same gateway, same auth — only
# the department claim decides.
#
# Cost: Free for the authz path; the upstream call uses a real key on allows.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

ADMIN_TOKEN="${TOKEN:-}"
if [[ -z "$ADMIN_TOKEN" && -f "$TOKEN_FILE" ]]; then
  ADMIN_TOKEN=$(cat "$TOKEN_FILE")
fi
if [[ -z "$ADMIN_TOKEN" ]]; then
  echo "ERROR: Run demo/00-preflight.sh first." >&2; exit 1
fi

ALICE_ID="4c127601-741a-4064-8d43-12b7d24158bf"
BOB_ID="7cd34e51-9313-4c3a-b851-773b4668e63a"

if [[ ! -f /tmp/mint_jwt.py ]]; then
  fail "/tmp/mint_jwt.py missing — run demo 09 first."; exit 1
fi
ALICE_JWT=$(python3 /tmp/mint_jwt.py alice)
BOB_JWT=$(python3 /tmp/mint_jwt.py bob)

header "DEMO 10: AUTHZ — DENY BY DEPARTMENT" "Same gateway, same auth — department decides"

# ─────────────────────────────────────────────────
subheader "STEP 1: alice (eng) -> gpt-4 -> expect 403"

cmd "POST $BASE_URL/v1/messages   (model=gpt-4, alice)"
HTTP_CODE=$(curl -s -o /tmp/authz_deny_resp.json -w "%{http_code}" \
  -X POST "$BASE_URL/v1/messages" \
  -H "Authorization: Bearer $ALICE_JWT" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","max_tokens":32,"messages":[{"role":"user","content":"ping"}]}')

if [[ "$HTTP_CODE" == "403" ]]; then
  pass "alice -> gpt-* denied (HTTP 403, department mismatch)"
else
  fail "expected 403, got $HTTP_CODE"
fi

# ─────────────────────────────────────────────────
subheader "STEP 2: Deny reason from response body"
echo ""
python3 -m json.tool < /tmp/authz_deny_resp.json 2>/dev/null | sed 's/^/  /' \
  || sed 's/^/  /' /tmp/authz_deny_resp.json

# ─────────────────────────────────────────────────
subheader "STEP 3: Governance audit row (deny)"
echo "  Most recent authz deny for alice:"
"$CLI" infra db query \
  "SELECT decision, policy, reason, evaluated_rules FROM governance_decisions WHERE user_id='$ALICE_ID' AND policy='authz' AND decision='deny' ORDER BY created_at DESC LIMIT 1" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | sed 's/^/  /'

# ─────────────────────────────────────────────────
subheader "STEP 4: bob (finance) -> gpt-4 -> expect 200"

cmd "POST $BASE_URL/v1/messages   (model=gpt-4, bob)"
BOB_CODE=$(curl -s -o /tmp/authz_bob_resp.json -w "%{http_code}" \
  -X POST "$BASE_URL/v1/messages" \
  -H "Authorization: Bearer $BOB_JWT" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","max_tokens":32,"messages":[{"role":"user","content":"ping"}]}')

if [[ "$BOB_CODE" == "200" ]]; then
  pass "bob -> gpt-* allowed (HTTP 200)"
else
  fail "expected 200 for bob, got $BOB_CODE"
fi

# ─────────────────────────────────────────────────
subheader "STEP 5: Side-by-side"
cat <<EOF
  ┌──────────────┬───────────────┬──────────┬──────────────────────────┐
  │ subject      │ model         │ result   │ reason                   │
  ├──────────────┼───────────────┼──────────┼──────────────────────────┤
  │ alice (eng)  │ claude-3-…    │  ALLOW   │ department=eng matched   │
  │ alice (eng)  │ gpt-4         │  DENY    │ department=eng not assn  │
  │ bob (finance)│ gpt-4         │  ALLOW   │ department=finance match │
  └──────────────┴───────────────┴──────────┴──────────────────────────┘
EOF
echo ""

header "NEXT: ./demo/governance/11-authz-flow-replay.sh"
