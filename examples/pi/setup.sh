#!/bin/bash
# Wire Pi (https://pi.dev) to the Enterprise Demo governance gateway.
#
# What this does:
#   1. Installs the Pi coding agent if missing (npm).
#   2. Mints a gateway JWT via `systemprompt admin session login` and extracts
#      the session_id claim (the gateway enforces x-session-id == token.session_id).
#   3. Writes the token to ~/.config/systemprompt-pi/ (chmod 600). The session
#      is not written: the governance extension reads the JWT's own session_id
#      claim, and a PAT caller mints one at POST /api/public/gateway/sessions.
#   4. Installs the `systemprompt` provider into ~/.pi/agent/models.json and
#      the branded theme into ~/.pi/agent/themes/.
#   5. Smoke-tests POST /v1/messages through the gateway.
#
# Usage:
#   examples/pi/setup.sh            full setup
#   examples/pi/setup.sh --refresh  re-mint the token/session only (JWTs expire)
#
# Portable across macOS and Linux: no grep -P, no head -n -1, no sed -i.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$HERE/../.." && pwd)"
CRED_DIR="$HOME/.config/systemprompt-pi"
PI_AGENT_DIR="$HOME/.pi/agent"
REFRESH_ONLY=false
[[ "${1:-}" == "--refresh" ]] && REFRESH_ONLY=true

say()  { printf '\033[36m==>\033[0m %s\n' "$*"; }
pass() { printf '\033[32m ok\033[0m %s\n' "$*"; }
die()  { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ── CLI binary resolution (same precedence as demo/_common.sh) ──
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  CLI="$(command -v systemprompt || true)"
fi
[[ -n "$CLI" && -x "$CLI" ]] || die "systemprompt CLI not found. Run: just build (or install systemprompt)"

# ── Base URL from the active profile; pin localhost -> 127.0.0.1 ──
# (localhost can resolve to [::1] while the server binds IPv4 only.)
PROFILE="${PROFILE:-local}"
PROFILE_YAML="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
BASE_URL="${BASE_URL:-}"
if [[ -z "$BASE_URL" && -f "$PROFILE_YAML" ]]; then
  BASE_URL=$(grep -E '^[[:space:]]*api_server_url:' "$PROFILE_YAML" | head -1 \
    | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
fi
BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
BASE_URL="${BASE_URL/localhost/127.0.0.1}"
BASE_URL="${BASE_URL%/}"

# ── 1. Install Pi ──
if ! $REFRESH_ONLY; then
  if command -v pi >/dev/null 2>&1; then
    pass "Pi already installed ($(pi --version 2>/dev/null | head -1 || echo unknown))"
  else
    say "Installing Pi coding agent"
    command -v npm >/dev/null 2>&1 || die "npm not found; install Node.js first"
    npm install -g --ignore-scripts @earendil-works/pi-coding-agent
    pass "Pi installed"
  fi
fi

# ── 2. Mint gateway token + session id ──
say "Minting a gateway token (profile: $PROFILE)"
TOKEN=$("$CLI" admin session login --token-only 2>/dev/null \
  | grep -oE '[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1 || true)
[[ -n "$TOKEN" ]] || die "could not mint a token — is the server running? (just start)"

_jwt_payload_b64() {
  local payload pad
  payload=$(printf '%s' "$1" | cut -d. -f2)
  pad=$(( (4 - ${#payload} % 4) % 4 ))
  printf '%s' "$payload"
  # Why: this is the function's last command, so a bare `&&` test would return
  # 1 whenever the payload needs no padding -- and under `set -e` with
  # `pipefail` that kills the caller's pipeline. Whether it fires depends on the
  # JWT's length, so it fails only for some tokens.
  if [[ $pad -gt 0 ]]; then printf '%.0s=' $(seq 1 $pad); fi
}
SESSION_ID=$(_jwt_payload_b64 "$TOKEN" | tr '_-' '/+' | base64 -d 2>/dev/null \
  | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
[[ -n "$SESSION_ID" ]] || die "could not extract session_id from the minted JWT"

mkdir -p "$CRED_DIR"
printf '%s' "$TOKEN" > "$CRED_DIR/token"
# The governance extension resolves the gateway from here rather than guessing
# a port, so a profile on a non-default port needs no extension edit.
printf '%s' "$BASE_URL" > "$CRED_DIR/base-url"
# An admin session JWT is already accepted by /hooks/govern (aud=api), so admin
# setup needs no separate credential. new-user.sh overwrites this with a
# user-scope plugin token, which is the one that actually gets denied.
printf '%s' "$TOKEN" > "$CRED_DIR/hook-token"
chmod 600 "$CRED_DIR/token" "$CRED_DIR/hook-token"
pass "Credentials written to $CRED_DIR (session $SESSION_ID)"

# ── 3. Install models.json + theme ──
if ! $REFRESH_ONLY; then
  command -v jq >/dev/null 2>&1 || die "jq is required (brew install jq / apt install jq)"
  mkdir -p "$PI_AGENT_DIR/themes"

  say "Installing the systemprompt provider into $PI_AGENT_DIR/models.json"
  # Point the provider at this deployment's gateway URL.
  # Pi appends /v1/messages itself, so baseUrl carries no /v1 suffix.
  PROVIDER=$(jq --arg url "$BASE_URL" '.providers.systemprompt.baseUrl = $url | .providers.systemprompt' "$HERE/models.json")
  if [[ -f "$PI_AGENT_DIR/models.json" ]]; then
    MERGED=$(jq --argjson p "$PROVIDER" '.providers.systemprompt = $p' "$PI_AGENT_DIR/models.json")
    printf '%s\n' "$MERGED" > "$PI_AGENT_DIR/models.json"
    pass "Merged into existing models.json"
  else
    jq --argjson p "$PROVIDER" -n '{providers: {systemprompt: $p}}' > "$PI_AGENT_DIR/models.json"
    pass "Created models.json"
  fi

  say "Installing the governance extension"
  mkdir -p "$PI_AGENT_DIR/extensions"
  cp "$HERE/extensions/governance.ts" "$PI_AGENT_DIR/extensions/systemprompt-governance.ts"
  pass "Tool + prompt gate installed (reload inside Pi with /reload)"

  say "Installing the branded theme"
  cp "$HERE/themes/systemprompt.json" "$PI_AGENT_DIR/themes/systemprompt.json"
  SETTINGS="$PI_AGENT_DIR/settings.json"
  if [[ -f "$SETTINGS" ]]; then
    if [[ "$(jq -r '.theme // empty' "$SETTINGS")" == "" ]]; then
      UPDATED=$(jq '.theme = "systemprompt"' "$SETTINGS")
      printf '%s\n' "$UPDATED" > "$SETTINGS"
    fi
  else
    printf '{ "theme": "systemprompt" }\n' > "$SETTINGS"
  fi
  pass "Theme installed (select later with /settings if you switch away)"
fi

# ── 4. Smoke test ──
say "Smoke test: POST $BASE_URL/v1/messages (claude-sonnet-4-6)"
RESP=$(curl -fsS -m 40 -X POST "$BASE_URL/v1/messages" \
  -H "Authorization: Bearer $TOKEN" \
  -H "x-session-id: $SESSION_ID" \
  -H "anthropic-version: 2023-06-01" -H "content-type: application/json" \
  -d '{"model":"claude-sonnet-4-6","max_tokens":32,"messages":[{"role":"user","content":"reply with exactly: pong"}]}') \
  || die "gateway smoke test failed — check: systemprompt infra logs view --level error --since 5m"
printf '%s' "$RESP" | jq -r '"    gateway replied: \(.content[0].text // .)"'
pass "Gateway accepted the request"

echo
say "Done. Start Pi with:  pi"
echo "    Switch models mid-session with /model (Ctrl+L)."
echo "    Watch governed traffic with: systemprompt infra logs request list --limit 10"
echo "    Tokens expire; re-auth any time with: examples/pi/setup.sh --refresh"
