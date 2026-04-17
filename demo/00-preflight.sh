#!/bin/bash
# PREFLIGHT CHECK & TOKEN ACQUISITION
# Run this before all other demos. It:
#   1. Locates the CLI binary
#   2. Checks all services are running (3 agents + 2 MCP servers)
#   3. Creates a local admin session (bypasses OAuth — uses cloud identity)
#   4. Fetches the dashboard to extract the SYSTEMPROMPT_TOKEN (plugin token)
#   5. Decodes both JWTs to show the typed claims and auth model
#   6. Saves the plugin token to demo/.token for subsequent demos
#
# Auth model:
#   Production login: Google OAuth or GitHub → callback → session created
#   Local dev login:  `admin session login` → reads cloud identity from
#     .systemprompt/credentials.json → looks up user in local DB →
#     generates JWT signed with local jwt_secret → returns session token
#
#   The platform issues two kinds of JWT, both signed by the same secret:
#
#   Admin session token (scope=admin, 24h):
#     CLI commands, dashboard cookie, full API access
#
#   Plugin token / SYSTEMPROMPT_TOKEN (scope=service, 365 days):
#     Claude Code hooks send this as Authorization: Bearer <token>
#     Events tracked with this token appear on the user's dashboard
#     Shown in the "Share & Install" widget at /admin/profile
#
#   Both share the same user_id (sub claim) so events from either
#   token appear under the same user in the dashboard.
#
# WHY RUST: JWT claims are a typed struct — UserId, SessionId, UserType,
#   RateLimitTier are all newtypes enforced at compile time. The signing
#   key is loaded from SecretsBootstrap which won't let the service start
#   without a jwt_secret. Token validation returns typed JwtClaims, not
#   a raw map — .sub is a UserId, not a String.
#
# Cost: Free

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: just build" >&2
  exit 1
fi

export RUST_LOG="${RUST_LOG:-warn}"

TOKEN_FILE="$SCRIPT_DIR/.token"
PROFILE="${PROFILE:-local}"
# Honour the port configured in the active profile.yaml so preflight works
# when setup-local was run with non-default ports. Override with BASE_URL env.
_preflight_base_url() {
  local profile_yaml="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
  if [[ -f "$profile_yaml" ]]; then
    local url
    url=$(grep -E '^[[:space:]]*api_server_url:' "$profile_yaml" | head -1 | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
    [[ -n "$url" && "$url" != "null" ]] && { echo "$url"; return; }
  fi
  echo "http://localhost:8080"
}
BASE_URL="${BASE_URL:-$(_preflight_base_url)}"

# Helper: decode JWT payload and print selected claims
decode_jwt() {
  echo "$1" | cut -d. -f2 | python3 -c "
import sys, base64, json, datetime
payload = sys.stdin.read().strip()
payload += '=' * (4 - len(payload) % 4)
claims = json.loads(base64.urlsafe_b64decode(payload))
for key in ['sub', 'iss', 'scope', 'user_type', 'session_id', 'rate_limit_tier']:
    if key in claims:
        print(f'    {key:20s}: {claims[key]}')
aud = claims.get('aud', [])
if aud:
    print(f'    {\"audiences\":20s}: {aud}')
exp = claims.get('exp', 0)
if exp:
    print(f'    {\"expires\":20s}: {datetime.datetime.fromtimestamp(exp).strftime(\"%Y-%m-%d %H:%M\")}')
"
}

echo ""
echo "=========================================="
echo "  PREFLIGHT CHECK & TOKEN ACQUISITION"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  STEP 1: Service health
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 1: Service health"
echo "------------------------------------------"
echo ""

# Use set +e since validation errors may cause non-zero exit
set +e
# Strip ANSI escape codes so grep matches reliably
STATUS_OUTPUT=$("$CLI" infra services status 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | grep -E "running|stopped|services|PID|Service Status")
STATUS_EXIT=$?
set -e

if [[ $STATUS_EXIT -ne 0 ]]; then
  # If grep found nothing, try a direct process check
  echo "  Checking processes directly..."
  echo ""
  HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8080/admin/" 2>/dev/null || echo "000")
  echo "  HTTP service: $HTTP_STATUS (http://localhost:8080)"
  "$CLI" infra services status 2>/dev/null | tail -10 || echo "  (service status unavailable)"
else
  echo "$STATUS_OUTPUT"
fi

echo ""

# ──────────────────────────────────────────────
#  STEP 2: Create local admin session
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 2: Create local admin session"
echo ""
echo "  How it works:"
echo "    1. CLI reads cloud identity from"
echo "       .systemprompt/credentials.json"
echo "    2. Looks up user in local PostgreSQL"
echo "    3. Generates JWT signed with local jwt_secret from"
echo "       .systemprompt/profiles/local/secrets.json"
echo "    4. Returns admin session token"
echo ""
echo "  In production, this step is Google/GitHub OAuth."
echo "  For local dev, the CLI shortcut uses your cloud identity."
echo "------------------------------------------"
echo ""

# Resolve admin email: ADMIN_EMAIL env > credentials.json user_email > default
CLOUD_EMAIL="${ADMIN_EMAIL:-}"
if [[ -z "$CLOUD_EMAIL" && -f "$PROJECT_DIR/.systemprompt/credentials.json" ]]; then
  CLOUD_EMAIL=$(python3 -c "import json; print(json.load(open('$PROJECT_DIR/.systemprompt/credentials.json')).get('user_email',''))" 2>/dev/null || true)
fi
CLOUD_EMAIL="${CLOUD_EMAIL:-admin@localhost.dev}"

# Try login first; if user not found, auto-create and retry
ADMIN_TOKEN=$("$CLI" admin session login --email "$CLOUD_EMAIL" --token-only --profile "$PROFILE" 2>/dev/null | tail -1)

if [[ -z "$ADMIN_TOKEN" || ! "$ADMIN_TOKEN" == eyJ* ]]; then
  echo "  Admin user not found — creating automatically..."
  echo ""

  # Create user and promote to admin
  "$CLI" admin users create --name "admin" --email "$CLOUD_EMAIL" --profile "$PROFILE" 2>&1 || true
  USER_ID=$("$CLI" admin users search "$CLOUD_EMAIL" --profile "$PROFILE" 2>/dev/null \
    | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
  if [[ -n "$USER_ID" ]]; then
    "$CLI" admin users role promote "$USER_ID" --profile "$PROFILE" 2>&1 || true
    echo "  Created admin user: $CLOUD_EMAIL ($USER_ID)"
    echo ""
  fi

  # Retry login
  ADMIN_TOKEN=$("$CLI" admin session login --email "$CLOUD_EMAIL" --token-only --profile "$PROFILE" 2>/dev/null | tail -1)

  if [[ -z "$ADMIN_TOKEN" || ! "$ADMIN_TOKEN" == eyJ* ]]; then
    echo "  ERROR: Could not obtain admin session token after user creation." >&2
    echo "  Is the database running? Try: just start" >&2
    exit 1
  fi
fi

echo "  Admin session token (${#ADMIN_TOKEN} chars):"
echo ""
decode_jwt "$ADMIN_TOKEN"
echo ""
echo "  scope=admin, user_type=admin — full platform access, 24h expiry."
echo ""

# ──────────────────────────────────────────────
#  STEP 3: Extract plugin token from dashboard
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 3: Extract SYSTEMPROMPT_TOKEN from dashboard"
echo ""
echo "  The admin token is used as a session cookie to fetch"
echo "  $BASE_URL/admin/profile — the same page you see in"
echo "  the browser. The plugin token is embedded in the"
echo "  'Share & Install' widget HTML."
echo "------------------------------------------"
echo ""

PLUGIN_TOKEN=$(curl -s -b "access_token=$ADMIN_TOKEN" "$BASE_URL/admin/profile" \
  | sed -n 's/.*data-copy="\(eyJ[^"]*\)".*/\1/p' | head -1)

if [[ -z "$PLUGIN_TOKEN" || ! "$PLUGIN_TOKEN" == eyJ* ]]; then
  echo "  WARNING: Could not extract plugin token from dashboard."
  echo "  Falling back to admin token (works for all demo endpoints)."
  PLUGIN_TOKEN="$ADMIN_TOKEN"
  echo ""
else
  echo "  Plugin token / SYSTEMPROMPT_TOKEN (${#PLUGIN_TOKEN} chars):"
  echo ""
  decode_jwt "$PLUGIN_TOKEN"
  echo ""
  echo "  scope=service — this is what Claude Code hooks send."
  echo "  session_id=plugin_cowork-bundle — identifies the plugin."
  echo "  365-day expiry — long-lived for unattended hook calls."
  echo ""
fi

# ──────────────────────────────────────────────
#  STEP 4: Compare the two tokens
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 4: Two tokens, same user, same secret"
echo "------------------------------------------"
echo ""
echo "  ┌──────────────────┬──────────────────┬──────────────────────┐"
echo "  │                  │ Admin session     │ Plugin token         │"
echo "  ├──────────────────┼──────────────────┼──────────────────────┤"
echo "  │ scope            │ admin            │ service              │"
echo "  │ user_type        │ admin            │ service              │"
echo "  │ expiry           │ 24 hours         │ 365 days             │"
echo "  │ session_id       │ sess_<uuid>      │ plugin_cowork-bundle │"
echo "  │ used by          │ CLI, dashboard   │ Claude Code hooks    │"
echo "  │ sub (user_id)    │ same             │ same                 │"
echo "  │ iss (issuer)     │ same             │ same                 │"
echo "  │ signing key      │ same jwt_secret  │ same jwt_secret      │"
echo "  └──────────────────┴──────────────────┴──────────────────────┘"
echo ""
echo "  Both tokens are signed by the jwt_secret in:"
echo "    .systemprompt/profiles/local/secrets.json"
echo ""
echo "  Both carry the same user_id (sub claim), so events from"
echo "  either token appear under the same user in the dashboard."
echo ""
echo "  WHY RUST: Token generation uses SessionGenerator::new(secret, issuer)"
echo "  with a typed SessionParams struct. UserId, SessionId, UserType are"
echo "  newtypes — the compiler prevents mixing them up. The jsonwebtoken"
echo "  crate signs with HS256. All claims are validated on every request"
echo "  via validate_jwt_token() which returns typed JwtClaims, not a map."
echo ""

# ──────────────────────────────────────────────
#  STEP 5: Save token for subsequent demos
# ──────────────────────────────────────────────
echo "$PLUGIN_TOKEN" > "$TOKEN_FILE"

echo "------------------------------------------"
echo "  STEP 5: Token saved → demo/.token"
echo "------------------------------------------"
echo ""
echo "  All subsequent demos read this file automatically."
echo "  No need to pass TOKEN as an argument."
echo ""

echo "=========================================="
echo "  PREFLIGHT COMPLETE"
echo ""
echo "  Services: running"
echo "  Token: acquired and saved"
echo "  Dashboard: $BASE_URL/admin/"
echo ""
echo "  Next: ./demo/governance/01-happy-path.sh"
echo "=========================================="
