#!/bin/bash
# Select (or create) the user Pi acts as, and wire Pi to the gateway as them.
#
#   examples/pi/new-user.sh [email] [display-name]
#
# With no arguments it lists the users already in the database (admins first)
# and asks which one to act as, so a run's sessions, requests, costs, and
# governance decisions land on a real profile page instead of a synthetic
# pi-demo account. Pass an email to skip the menu; `n` in the menu creates a
# new demo user the old way.
#
# What this does:
#   1. Resolves the user — picked from the database, or created on request.
#   2. Issues a personal access token (sp-live-…) FOR that user via the
#      admin API — the same credential the /admin/devices page self-issues.
#      The gateway accepts PATs directly (x-api-key or Bearer) and resolves
#      the user's roles live from the DB on every request.
#   3. Mints a plugin JWT for the governance extension — the PAT works on
#      /v1/messages but /hooks/govern validates a JWT and rejects it. Roles are
#      never mutated for a user this script did not create.
#   4. Writes both credentials, plus the acting identity (user.json), to
#      ~/.config/systemprompt-pi/ so the Pi provider and the governance
#      extension installed by setup.sh pick them up unchanged.
#   5. Mints a gateway session for the PAT and smoke-tests POST /v1/messages.
#      The gateway attests x-session-id against a session row it issued, so a
#      PAT caller mints one rather than inventing a label.
#
# Portable across macOS and Linux: no grep -P, no head -n -1, no sed -i.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$HERE/../.." && pwd)"
CRED_DIR="$HOME/.config/systemprompt-pi"

say()  { printf '\033[36m==>\033[0m %s\n' "$*"; }
pass() { printf '\033[32m ok\033[0m %s\n' "$*"; }
die()  { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

EMAIL="${1:-}"
NAME="${2:-}"

CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
[[ -x "$CLI" ]] || CLI="$(command -v systemprompt || true)"
[[ -n "$CLI" && -x "$CLI" ]] || die "systemprompt CLI not found. Run: just build"

PROFILE="${PROFILE:-local}"
PROFILE_YAML="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
BASE_URL="${BASE_URL:-}"
if [[ -z "$BASE_URL" && -f "$PROFILE_YAML" ]]; then
  BASE_URL=$(grep -E '^[[:space:]]*api_server_url:' "$PROFILE_YAML" | head -1 \
    | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
fi
BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
BASE_URL="${BASE_URL%/}"
# Two URLs, deliberately. curl gets 127.0.0.1 because `server.host: localhost`
# resolves to [::1] on some restarts and the connection is refused. The browser
# gets the configured host untouched: the dashboard session cookie is set
# without a Domain attribute (callback.rs:126), so it is host-only — a link to
# 127.0.0.1 is a different cookie jar from a login at localhost and bounces the
# operator to the login page.
DASHBOARD_URL="$BASE_URL"
BASE_URL="${BASE_URL/localhost/127.0.0.1}"

# ── 1. Resolve the user ──
# shellcheck source=../../scripts/select-user.sh
source "$PROJECT_DIR/scripts/select-user.sh"

if [[ -n "$EMAIL" ]]; then
  # An email argument names an existing user. If nothing matches, create it —
  # that is the documented non-interactive path for provisioning a fresh demo
  # user, and only a user created here may have its roles touched later.
  case "$EMAIL" in *@*.*) ;; *) die "'$EMAIL' does not look like an email address" ;; esac
  if ! select_db_user "$EMAIL" 2>/dev/null; then
    say "No user $EMAIL yet — creating"
    _sel_create_user "$EMAIL" "${NAME:-Pi Demo}" || die "could not create $EMAIL"
  fi
else
  say "Choosing the user Pi will act as"
  select_db_user || die "no user selected"
fi

NEW_USER_ID="$SEL_USER_ID"
EMAIL="$SEL_USER_EMAIL"
NAME="${SEL_USER_NAME:-$EMAIL}"
pass "Acting as $EMAIL ($NEW_USER_ID), roles: $SEL_USER_ROLES"

# ── 2. Issue a PAT for the user via the admin API ──
say "Issuing a gateway API key for $EMAIL"
ADMIN_TOKEN=$("$CLI" admin session login --token-only 2>/dev/null \
  | grep -oE '[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1)
[[ -n "$ADMIN_TOKEN" ]] || die "could not mint an admin session — is the server running? (just start)"

PAT=$(curl -fsS -X POST "$BASE_URL/api/public/admin/users/$NEW_USER_ID/pats" \
  -H "Authorization: Bearer $ADMIN_TOKEN" -H "content-type: application/json" \
  -d "{\"name\":\"pi-demo $(date -u +%Y%m%dT%H%M%SZ)\"}" | sed -n 's/.*"secret":"\([^"]*\)".*/\1/p')
[[ -n "$PAT" ]] || die "PAT issuance failed (needs the /users/{id}/pats admin endpoint — rebuild + restart?)"
pass "API key issued (${PAT:0:12}…)"

# ── 3. Issue the governance credential ──
#
# Two credentials, two endpoints:
#   /v1/messages   accepts the sp-live-… PAT above.
#   /hooks/govern  validates a JWT (aud = hook|plugin|api) and rejects a PAT,
#                  so the Pi governance extension needs its own token.
#
# `admin keys issue-plugin-token` refuses non-admins, and governance resolves
# access scope from the caller's LIVE DB role rather than from the token. For a
# user created here that means promote, mint, demote — the token stays valid and
# the next decision reads role `user`, which is what makes the scope_check and
# tool_blocklist denials real instead of narrated. For a user picked from the
# database, ensure_plugin_token refuses to touch their roles at all.
say "Minting a governance token for $EMAIL"
HOOK_TOKEN=$(ensure_plugin_token "$NEW_USER_ID" "$EMAIL") \
  || die "no governance token for $EMAIL — the tool gate would fail open"
if [[ "$SEL_USER_IS_ADMIN" == "1" ]]; then
  pass "Governance token issued; roles untouched ($SEL_USER_ROLES)"
  echo "    Note: admins are exempt from scope_check and tool_blocklist, so"
  echo "    demo/governance/09-pi-agent.sh will show those two cases allowed."
else
  pass "Governance token issued; DB role is 'user'"
fi

# ── 4. Write Pi credentials ──
mkdir -p "$CRED_DIR"
printf '%s' "$PAT" > "$CRED_DIR/token"
printf '%s' "$HOOK_TOKEN" > "$CRED_DIR/hook-token"
printf '%s' "$BASE_URL" > "$CRED_DIR/base-url"
# The acting identity, so the demos can name the caller and derive the right
# governance expectations without re-querying the database.
printf '{"id":"%s","email":"%s","name":"%s","roles":"%s","is_admin":%s}\n' \
  "$NEW_USER_ID" "$EMAIL" "$NAME" "$SEL_USER_ROLES" \
  "$([[ "$SEL_USER_IS_ADMIN" == "1" ]] && echo true || echo false)" \
  > "$CRED_DIR/user.json"
chmod 600 "$CRED_DIR/token" "$CRED_DIR/hook-token"
pass "Pi credentials written to $CRED_DIR"

# ── 5. Mint a session, then smoke test as the selected user ──
say "Minting a gateway session for the PAT"
SESSION_ID=$(curl -fsS -X POST "$BASE_URL/api/public/gateway/sessions" \
  -H "x-api-key: $PAT" | sed -n 's/.*"session_id":"\([^"]*\)".*/\1/p')
[[ -n "$SESSION_ID" ]] || die "session mint failed at POST $BASE_URL/api/public/gateway/sessions"
# Recorded so trace.sh and the walkthrough can point at the same timeline the
# smoke test below lands in, rather than minting a second unrelated session.
printf '%s' "$SESSION_ID" > "$CRED_DIR/session"
pass "Session $SESSION_ID issued"

say "Smoke test: POST $BASE_URL/v1/messages as $EMAIL"
RESP=$(curl -fsS -m 40 -X POST "$BASE_URL/v1/messages" \
  -H "x-api-key: $PAT" -H "x-session-id: $SESSION_ID" \
  -H "anthropic-version: 2023-06-01" -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-4-6","max_tokens":32,"messages":[{"role":"user","content":"reply with exactly: pong"}]}') \
  || die "gateway smoke test failed — check: systemprompt infra logs view --level error --since 5m"
printf '%s' "$RESP" | sed -n 's/.*"text":"\([^"]*\)".*/    gateway replied: \1/p'
pass "Gateway accepted the request as $EMAIL"

echo
say "Done. Pi now acts as $EMAIL. Start it with:  pi"
echo "    (run examples/pi/setup.sh first if Pi itself is not installed yet)"
echo
say "Evidence — open these as an admin:"
echo "    profile + usage   $DASHBOARD_URL/admin/user?id=$NEW_USER_ID"
echo "    model access      $DASHBOARD_URL/admin/models?user_id=$NEW_USER_ID"
echo "    this session      $DASHBOARD_URL/admin/sessions/$SESSION_ID"
echo "    governed timeline $DASHBOARD_URL/admin/demo/trace?session=$SESSION_ID"
echo "    all requests      $DASHBOARD_URL/admin/requests"
echo
echo "    Send a prompt of your own and watch it land:"
echo "      examples/pi/trace.sh \"your prompt here\""
