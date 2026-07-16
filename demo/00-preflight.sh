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
#   Admin session token (scope=admin, user_type=admin, 24h):
#     CLI commands, dashboard cookie, full API access
#
#   Plugin token / SYSTEMPROMPT_TOKEN (scope=admin, user_type=admin, 365d):
#     Claude Code hooks send this as Authorization: Bearer <token>.
#     NOTE: in core v0.11.0, `admin keys issue-plugin-token` hardcodes
#     Permission::Admin, so user_type derives to Admin (not Service).
#     aud=[api,plugin] because `resource: Some("plugin")` is merged into
#     the aud array in domain/oauth/services/generation.rs:96-99. The
#     session_id is a fresh sess_<uuid> (NOT plugin_<id>) — the plugin
#     identity is carried by the separate plugin_id claim.
#     Tracked as tech debt in systemprompt-core/issues.md.
#
#   Both tokens are minted for the SAME --email so the dashboard
#   groups events under one user (sub claim).
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
  # No cargo target dir (container / installed-binary deployments): use PATH.
  CLI="$(command -v systemprompt || true)"
fi
if [[ -z "$CLI" || ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: just build (or install systemprompt)" >&2
  exit 1
fi

export RUST_LOG="${RUST_LOG:-warn}"

TOKEN_FILE="$SCRIPT_DIR/.token"
# Match demo/_common.sh: prefer local, fall back to the container profile.
if [[ -z "${PROFILE:-}" ]]; then
  if [[ ! -d "$PROJECT_DIR/.systemprompt/profiles/local" && -d "$PROJECT_DIR/.systemprompt/profiles/docker" ]]; then
    PROFILE=docker
  else
    PROFILE=local
  fi
fi
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
#  STEP 0: Cloud credentials check
# ──────────────────────────────────────────────
# The CLI prints a WARN line on every invocation when cloud creds are
# expired (or missing). Detect it up-front so the operator sees one clear
# actionable line here, and downstream cloud-dependent demos can skip.
# Demos that don't need cloud are unaffected — local profile keeps working.
CLOUD_OFFLINE=0
set +e
WHOAMI_OUTPUT=$("$CLI" cloud auth whoami 2>&1)
set -e
if printf '%s' "$WHOAMI_OUTPUT" | grep -qiE 'token (status:[[:space:]]*)?expired|credentials unavailable|not authenticated'; then
  CLOUD_OFFLINE=1
  CRED_FILE="$PROJECT_DIR/.systemprompt/credentials.json"
  EXP_NOTE=""
  if [[ -f "$CRED_FILE" ]]; then
    EXP_NOTE=$(python3 -c "
import json, base64, datetime, sys
try:
    tok = json.load(open('$CRED_FILE')).get('api_token','')
    payload = tok.split('.')[1]
    payload += '=' * (4 - len(payload) % 4)
    claims = json.loads(base64.urlsafe_b64decode(payload))
    exp = claims.get('exp', 0)
    if exp:
        print(datetime.datetime.fromtimestamp(exp).strftime('%Y-%m-%d'))
except Exception:
    pass
" 2>/dev/null || true)
  fi
  echo "  NOTE: Cloud credentials expired${EXP_NOTE:+ ($EXP_NOTE)}."
  echo "        Local-profile demos will continue normally."
  echo "        Run 'systemprompt cloud auth login' if you need cloud sync."
  echo ""
fi
export CLOUD_OFFLINE

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
  HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/admin/" 2>/dev/null || echo "000")
  echo "  HTTP service: $HTTP_STATUS (http://localhost:8080)"
  "$CLI" infra services status 2>/dev/null | tail -10 || echo "  (service status unavailable)"
else
  echo "$STATUS_OUTPUT"
fi

echo ""

# The admin session login (Step 2) and dashboard fetch (Step 3) both require
# the HTTP API server. Probe it now so a stopped platform fails here with a
# clear message instead of surfacing later as a misleading "user not found".
# curl prints `000` as the status on a failed connection; `|| true` keeps
# `set -e` happy without appending a second value to the captured output.
API_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/admin/" 2>/dev/null || true)
if [[ -z "$API_HEALTH" || "$API_HEALTH" == "000" ]]; then
  echo "  ERROR: API server unreachable at $BASE_URL." >&2
  echo "  The database may be up, but the HTTP server is not running." >&2
  echo "  Start the platform first:  just start" >&2
  exit 1
fi
echo "  API server: reachable ($BASE_URL -> HTTP $API_HEALTH)"
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

# Extract a bare JWT from CLI output. `--token-only` still emits a
# `[profile: …]` banner line, so a blind `tail -1` can capture the banner
# instead of the token — match the JWT shape (three base64url segments).
_extract_jwt() {
  grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1
}

# Try login first; if (and only if) the user is missing, auto-create and retry.
# `|| true` keeps `set -e` from aborting here: a missing user is an expected
# first-run state handled by the auto-create block below.
LOGIN_OUTPUT=$("$CLI" admin session login --email "$CLOUD_EMAIL" --token-only --profile "$PROFILE" 2>&1 || true)
ADMIN_TOKEN=$(printf '%s\n' "$LOGIN_OUTPUT" | _extract_jwt)

if [[ -z "$ADMIN_TOKEN" ]]; then
  # A connection failure is not a missing user — creating the user would not
  # help, and the duplicate-key noise only obscures the real cause.
  if printf '%s' "$LOGIN_OUTPUT" | grep -qiE 'connection refused|tcp connect error|error sending request|connect error'; then
    echo "  ERROR: API server unreachable at $BASE_URL." >&2
    echo "  Start the platform first:  just start" >&2
    exit 1
  fi

  echo "  Admin user not found — creating automatically..."
  echo ""

  # Create the user and promote to admin. `users create` is idempotent for
  # the demo's purposes: a duplicate-email error just means the user already
  # exists — which is success — and the search + promote below still apply.
  "$CLI" admin users create --name "admin" --email "$CLOUD_EMAIL" --profile "$PROFILE" 2>&1 || true
  USER_ID=$("$CLI" admin users search "$CLOUD_EMAIL" --profile "$PROFILE" 2>/dev/null \
    | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
  if [[ -n "$USER_ID" ]]; then
    "$CLI" admin users role promote "$USER_ID" --profile "$PROFILE" 2>&1 || true
    echo "  Admin user ready: $CLOUD_EMAIL ($USER_ID)"
    echo ""
  else
    echo "  ERROR: Could not create or locate admin user $CLOUD_EMAIL." >&2
    exit 1
  fi

  # Retry login now that the user exists.
  LOGIN_OUTPUT=$("$CLI" admin session login --email "$CLOUD_EMAIL" --token-only --profile "$PROFILE" 2>&1 || true)
  ADMIN_TOKEN=$(printf '%s\n' "$LOGIN_OUTPUT" | _extract_jwt)

  if [[ -z "$ADMIN_TOKEN" ]]; then
    echo "  ERROR: Could not obtain an admin session token for $CLOUD_EMAIL." >&2
    echo "  The login command reported:" >&2
    printf '%s\n' "$LOGIN_OUTPUT" | sed 's/^/    /' >&2
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
echo "  STEP 3: Mint SYSTEMPROMPT_TOKEN via admin keys issue-plugin-token"
echo ""
echo "  Calls the CLI subcommand that lands in core 0.11:"
echo "    systemprompt admin keys issue-plugin-token --token-only \\"
echo "      --email $CLOUD_EMAIL --profile $PROFILE"
echo ""
echo "  Mints an RS256 JWT, 365-day expiry. The aud claim ends up as"
echo "  [api, plugin] because the handler sets audience=[Api] AND"
echo "  resource=Some(\"plugin\"); the JWT builder merges resource into"
echo "  aud (domain/oauth/services/generation.rs:96-99). The plugin"
echo "  identity is also carried explicitly via the plugin_id claim."
echo ""
echo "  Caveat: the handler hardcodes Permission::Admin (issue_plugin_token.rs:91-97),"
echo "  so user_type derives to Admin — see systemprompt-core/issues.md."
echo "------------------------------------------"
echo ""

PLUGIN_TOKEN=$("$CLI" admin keys issue-plugin-token --token-only \
  --email "$CLOUD_EMAIL" --profile "$PROFILE" 2>/dev/null | tail -1)

if [[ -z "$PLUGIN_TOKEN" || ! "$PLUGIN_TOKEN" == eyJ* ]]; then
  echo "  ERROR: 'admin keys issue-plugin-token' did not return a JWT." >&2
  echo "  Re-run manually for diagnostics:" >&2
  echo "    $CLI admin keys issue-plugin-token --email $CLOUD_EMAIL --profile $PROFILE" >&2
  exit 1
fi

echo "  Plugin token / SYSTEMPROMPT_TOKEN (${#PLUGIN_TOKEN} chars):"
echo ""
decode_jwt "$PLUGIN_TOKEN"
echo ""
echo "  aud=[api,plugin], plugin_id=cowork-bundle — what Claude Code hooks send."
echo "  365-day expiry — long-lived for unattended hook calls."
echo ""

# ──────────────────────────────────────────────
#  STEP 4: Compare the two tokens
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  STEP 4: Two tokens, same user, same secret"
echo "------------------------------------------"
echo ""
echo "  ┌──────────────────┬──────────────────────┬──────────────────────────┐"
echo "  │                  │ Admin session        │ Plugin token             │"
echo "  ├──────────────────┼──────────────────────┼──────────────────────────┤"
echo "  │ scope            │ admin                │ admin                    │"
echo "  │ user_type        │ admin                │ admin (derived from      │"
echo "  │                  │                      │   hardcoded Perm::Admin) │"
echo "  │ expiry           │ 24 hours             │ 365 days                 │"
echo "  │ session_id       │ sess_<uuid>          │ sess_<uuid> (fresh)      │"
echo "  │ aud              │ [web,api,a2a,mcp]    │ [api, plugin]            │"
echo "  │ plugin_id        │ —                    │ cowork-bundle            │"
echo "  │ used by          │ CLI, dashboard       │ Claude Code hooks        │"
echo "  │ sub (user_id)    │ same                 │ same                     │"
echo "  │ iss (issuer)     │ same                 │ same                     │"
echo "  │ signing key      │ same jwt_secret      │ same jwt_secret          │"
echo "  └──────────────────┴──────────────────────┴──────────────────────────┘"
echo ""
echo "  Both rows above are taken from the live JWTs decoded in Steps 2-3."
echo "  Plugin-identity is carried by the plugin_id claim, not session_id."
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
echo "  crate signs with RS256 (kid-pinned). All claims are validated on every request"
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

# ──────────────────────────────────────────────
#  STEP 6: Provision a USER-SCOPE token for the deny demos
# ──────────────────────────────────────────────
# The admin token above is allowed by scope_check and tool_blocklist (admins are
# exempt), so it cannot prove a genuine deny. Governance derives scope from the
# caller's LIVE DB roles, not the agent_id in the payload — so we mint a plugin
# token for a dedicated `demo_user` while it is admin (issue-plugin-token refuses
# non-admins), then demote it to `user`. The token stays valid; the next
# governance decision reads the DB role and resolves it to User scope. This is the
# same recipe the `manage_permissions` skill documents.
USER_TOKEN_FILE="$SCRIPT_DIR/.token.user"
USER_EMAIL="${DEMO_USER_EMAIL:-demo_user@demo.local}"

echo "------------------------------------------"
echo "  STEP 6: Provision user-scope token → demo/.token.user"
echo ""
echo "  Mints a plugin token for $USER_EMAIL (admin → token → user)."
echo "  Governance reads DB roles live, so this token resolves to User"
echo "  scope and the deny demos can prove a real scope_check / blocklist"
echo "  denial instead of narrating one."
echo "------------------------------------------"
echo ""

"$CLI" admin users create --name "demo_user" --email "$USER_EMAIL" --if-not-exists --profile "$PROFILE" 2>&1 \
  | grep -viE '^\[profile|already exists' || true
# The search output is a rendered card (id on its own line), not JSON, so match
# the UUID shape directly — works regardless of the human/JSON output layer.
DEMO_USER_ID=$("$CLI" admin users search "$USER_EMAIL" --profile "$PROFILE" 2>/dev/null \
  | grep -oiE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)

if [[ -z "$DEMO_USER_ID" ]]; then
  echo "  WARNING: could not locate $USER_EMAIL; skipping user-scope token." >&2
  echo "  The deny demos (02, 05) need demo/.token.user — re-run preflight once" >&2
  echo "  the user database is reachable." >&2
  echo ""
else
  # Promote so a plugin token can be minted, capture the token, then demote so the
  # live DB role is `user`. The token's authority follows the DB role, not the
  # role at mint time.
  "$CLI" admin users role promote "$DEMO_USER_ID" --profile "$PROFILE" >/dev/null 2>&1 || true
  USER_TOKEN=$("$CLI" admin keys issue-plugin-token --token-only \
    --email "$USER_EMAIL" --profile "$PROFILE" 2>/dev/null | _extract_jwt)
  "$CLI" admin users role demote "$DEMO_USER_ID" --profile "$PROFILE" >/dev/null 2>&1 || true

  if [[ -z "$USER_TOKEN" ]]; then
    echo "  WARNING: could not mint a plugin token for $USER_EMAIL." >&2
    echo "  The deny demos will fall back and report a missing user token." >&2
    echo ""
  else
    echo "$USER_TOKEN" > "$USER_TOKEN_FILE"
    echo "  User-scope token (${#USER_TOKEN} chars) for $USER_EMAIL ($DEMO_USER_ID):"
    echo ""
    decode_jwt "$USER_TOKEN"
    echo ""
    echo "  DB role is now 'user' — scope_check denies mcp__systemprompt__* and"
    echo "  tool_blocklist denies destructive tools for this token. Saved to"
    echo "  demo/.token.user (gitignored like demo/.token)."
    echo ""
  fi
fi

echo "=========================================="
echo "  PREFLIGHT COMPLETE"
echo ""
echo "  Services: running"
echo "  Token: acquired and saved"
echo "  Dashboard: $BASE_URL/admin/"
echo ""
echo "  Next: ./demo/governance/01-happy-path.sh"
echo "=========================================="
