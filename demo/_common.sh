#!/bin/bash
# Shared helper for all demo scripts.
# Source this at the top of every demo:
#   source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"
# Or from top-level scripts:
#   source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

# Find DEMO_ROOT by walking up until we find _common.sh
_find_demo_root() {
  local dir="$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)"
  while [[ "$dir" != "/" ]]; do
    if [[ -f "$dir/_common.sh" ]]; then
      echo "$dir"
      return
    fi
    dir="$(dirname "$dir")"
  done
  echo "$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)"
}

DEMO_ROOT="$(_find_demo_root)"
PROJECT_DIR="$(dirname "$DEMO_ROOT")"

# Suppress verbose Rust logging — show warnings and errors only
export RUST_LOG="${RUST_LOG:-warn}"

# ── CLI binary resolution ──────────────────────
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: just build" >&2
  exit 1
fi

# ── Token loading ──────────────────────────────
TOKEN_FILE="$DEMO_ROOT/.token"
USER_TOKEN_FILE="$DEMO_ROOT/.token.user"
PROFILE="${PROFILE:-local}"
# Derive BASE_URL from the active profile so demos work when setup-local was
# invoked with non-default ports. Precedence: BASE_URL env > profile.yaml
# api_server_url > localhost:8080.
_derive_base_url() {
  local profile_yaml="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
  if [[ -f "$profile_yaml" ]]; then
    local url
    url=$(grep -E '^[[:space:]]*api_server_url:' "$profile_yaml" | head -1 | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
    [[ -n "$url" && "$url" != "null" ]] && { echo "$url"; return; }
  fi
  echo "http://localhost:8080"
}
BASE_URL="${BASE_URL:-$(_derive_base_url)}"

load_token() {
  # Precedence: explicit arg > exported $TOKEN env > demo/.token file. The env
  # path lets an orchestrator (e.g. scaled/run.sh) inject a token signed by a
  # different profile's secret without clobbering the on-disk local token.
  TOKEN="${1:-${TOKEN:-}}"
  if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
    TOKEN=$(cat "$TOKEN_FILE")
  fi
  if [[ -z "$TOKEN" ]]; then
    echo ""
    echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument."
    echo ""
    exit 1
  fi
}

# Load the USER-SCOPE plugin token (demo/.token.user, minted by 00-preflight.sh
# for demo_user@demo.local, whose DB role is `user`). Governance derives scope
# from the caller's live DB roles, so this token resolves to User scope and the
# scope_check / tool_blocklist policies genuinely DENY admin-only and destructive
# tools. The admin demo/.token would be allowed by those same policies — which is
# exactly why the deny demos must use this token to be honest.
# Precedence: explicit arg > exported $USER_TOKEN env > demo/.token.user file.
load_user_token() {
  USER_TOKEN="${1:-${USER_TOKEN:-}}"
  if [[ -z "$USER_TOKEN" && -f "$USER_TOKEN_FILE" ]]; then
    USER_TOKEN=$(cat "$USER_TOKEN_FILE")
  fi
  if [[ -z "$USER_TOKEN" ]]; then
    echo ""
    echo "  No user-scope token. Run ./demo/00-preflight.sh first"
    echo "  (it provisions demo/.token.user), or pass one as the first argument."
    echo ""
    exit 1
  fi
}

# Assert that a /hooks/govern JSON response carries the expected permissionDecision.
# Prints a GREEN PASS line on a match; prints a RED FAIL line and exits non-zero on
# a mismatch. This is what makes the governance demos self-testing: if the backend
# stops denying (e.g. a deny demo is accidentally pointed at an admin token), the
# script fails loudly instead of narrating a fiction.
#   assert_decision "<response_json>" "<expected>" "<label>"
assert_decision() {
  local response="$1" expected="$2" label="$3"
  local actual
  actual=$(printf '%s' "$response" | python3 -c \
    "import sys, json
try:
    d = json.load(sys.stdin)
    # The govern hook nests the decision under hookSpecificOutput; fall back to a
    # top-level field for any simpler response shape.
    hso = d.get('hookSpecificOutput') or {}
    print(hso.get('permissionDecision') or d.get('permissionDecision', ''))
except Exception:
    print('')" 2>/dev/null)
  if [[ "$actual" == "$expected" ]]; then
    echo -e "  ${GREEN}✓ PASS${R} — $label: permissionDecision=${actual}"
  else
    echo -e "  ${RED}✗ FAIL${R} — $label: expected permissionDecision=${expected}, got '${actual:-<none>}'" >&2
    echo "    Response was: $response" >&2
    exit 1
  fi
}

# ── Structured-data + validation helpers ──────
# In 0.15.0 every list/query command renders a human box-table by default and
# only emits machine-readable JSON under `--json` (tables: {columns, items[]};
# cards: {sections[]}). These helpers force the structured path and assert on
# it, so a demo fails loudly instead of narrating fiction when data is missing.

# jq powers every structured assertion below. Fail early and clearly rather
# than letting parses silently yield empty.
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: 'jq' is required by the demo suite." >&2
  echo "       macOS: brew install jq   Debian/Ubuntu: sudo apt-get install -y jq" >&2
  exit 1
fi

# Run a CLI subcommand in --json mode, emitting raw JSON to stdout.
#   cli_json infra jobs list
cli_json() {
  "$CLI" --json "$@" --profile "$PROFILE" 2>/dev/null
}

# Extract a scalar via jq from a --json CLI call (empty string if absent).
#   id=$(json_first '.items[0].id' admin users list)
json_first() {
  local path="$1"; shift
  cli_json "$@" | jq -r "${path} // empty" 2>/dev/null
}

# COUNT(*) (or first column of first row) from a SQL query, as a bare integer.
# Returns 0 on any failure. Always uses --json so it survives table-format changes.
#   n=$(db_count "SELECT COUNT(*) FROM governance_decisions")
db_count() {
  local n
  n=$(cli_json infra db query "$1" | jq -r '(.items[0] | to_entries[0].value) // 0' 2>/dev/null)
  [[ "$n" =~ ^[0-9]+$ ]] && echo "$n" || echo 0
}

# Assert a numeric value is >= a minimum; PASS line on success, exit 1 on failure.
#   assert_min "$n" 1 "governance_decisions rows"
assert_min() {
  local actual="$1" min="$2" label="$3"
  if [[ "${actual:-}" =~ ^[0-9]+$ ]] && (( actual >= min )); then
    echo -e "  ${GREEN}✓ PASS${R} — $label: ${actual} (>= ${min})"
  else
    echo -e "  ${RED}✗ FAIL${R} — $label: expected >= ${min}, got '${actual:-<none>}'" >&2
    exit 1
  fi
}

# Assert two values are equal; PASS line on success, exit 1 on failure.
assert_eq() {
  local actual="$1" expected="$2" label="$3"
  if [[ "$actual" == "$expected" ]]; then
    echo -e "  ${GREEN}✓ PASS${R} — $label: ${actual}"
  else
    echo -e "  ${RED}✗ FAIL${R} — $label: expected ${expected}, got '${actual:-<none>}'" >&2
    exit 1
  fi
}

# Assert a value is non-empty (and not the literal "null"); exit 1 otherwise.
assert_nonempty() {
  local value="$1" label="$2"
  if [[ -n "$value" && "$value" != "null" ]]; then
    echo -e "  ${GREEN}✓ PASS${R} — $label"
  else
    echo -e "  ${RED}✗ FAIL${R} — $label: value was empty" >&2
    exit 1
  fi
}

# ── Colors ─────────────────────────────────────
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
DIM='\033[2m'
R='\033[0m'

# ── Output helpers ─────────────────────────────
header() {
  echo ""
  echo "=========================================="
  echo "  $1"
  if [[ -n "${2:-}" ]]; then
    echo "  $2"
  fi
  echo "=========================================="
  echo ""
}

subheader() {
  echo "------------------------------------------"
  echo "  $1"
  if [[ -n "${2:-}" ]]; then
    echo "  $2"
  fi
  echo "------------------------------------------"
  echo ""
}

step() {
  echo -e "  ${BOLD}$1${R}"
  echo ""
}

divider() {
  echo ""
  echo "──────────────────────────────────────────"
  echo ""
}

pass() {
  echo -e "  ${GREEN}✓ $1${R}"
}

fail() {
  echo -e "  ${RED}✗ $1${R}"
}

info() {
  echo -e "  ${CYAN}$1${R}"
}

warn() {
  echo -e "  ${YELLOW}$1${R}"
}

cost_note() {
  echo -e "  ${DIM}Cost: $1${R}"
}

cmd() {
  echo -e "  ${DIM}\$ $1${R}"
  echo ""
}

# ── CLI execution helper ──────────────────────
# Runs a CLI command, prints it, and captures output
run_cli() {
  local display_cmd="systemprompt $*"
  cmd "$display_cmd"
  "$CLI" "$@" --profile "$PROFILE" 2>&1
}

# Runs a CLI command and indents output
run_cli_indented() {
  local display_cmd="systemprompt $*"
  cmd "$display_cmd"
  "$CLI" "$@" --profile "$PROFILE" 2>&1 | sed 's/^/  /'
}

# ── Load-testing helper ───────────────────────
# Installs the `hey` HTTP benchmarking tool into $HEY (default /tmp/hey),
# picking the binary that matches this host. Callers: load-test demos.
#
# Supported:  Darwin/x86_64, Darwin/arm64 (via Rosetta 2), Linux/x86_64.
# Unsupported: Linux/arm64 and everything else — prints a clear install hint.
install_hey() {
  # Prefer system-installed hey over /tmp/hey
  if command -v hey >/dev/null 2>&1; then
    HEY="$(command -v hey)"
    return 0
  fi
  HEY="${HEY:-/tmp/hey}"
  # Re-use a cached binary only if it actually executes on this host
  # (a stale hey_linux_amd64 left over on a Mac is the common failure mode).
  if [[ -x "$HEY" ]] && "$HEY" --help >/dev/null 2>&1; then
    return 0
  fi
  rm -f "$HEY"
  local os arch url
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os/$arch" in
    Darwin/*)
      url="https://hey-release.s3.us-east-2.amazonaws.com/hey_darwin_amd64"
      echo "  Installing hey from $url ..."
      if ! curl -fsSL "$url" -o "$HEY"; then
        echo "ERROR: failed to download hey. Install with: brew install hey" >&2
        return 1
      fi
      chmod +x "$HEY"
      if ! "$HEY" --help >/dev/null 2>&1; then
        echo "ERROR: downloaded hey cannot execute on $os/$arch." >&2
        if [[ "$arch" == "arm64" ]]; then
          echo "       Apple Silicon needs Rosetta 2: 'softwareupdate --install-rosetta'" >&2
          echo "       or a native build: 'brew install hey'" >&2
        fi
        rm -f "$HEY"
        return 1
      fi
      ;;
    Linux/x86_64|Linux/amd64)
      # Try package manager first (most reliable on Linux)
      if command -v apt-get >/dev/null 2>&1; then
        echo "  Installing hey via apt-get ..."
        if sudo apt-get install -y hey >/dev/null 2>&1; then
          HEY="$(command -v hey)"
          return 0
        fi
      fi
      # Fall back to S3 binary
      url="https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64"
      echo "  Installing hey from $url ..."
      if ! curl -fsSL "$url" -o "$HEY"; then
        echo "ERROR: failed to download hey. Install with: sudo apt-get install hey" >&2
        echo "       or: go install github.com/rakyll/hey@latest" >&2
        return 1
      fi
      chmod +x "$HEY"
      if ! "$HEY" --help >/dev/null 2>&1; then
        echo "ERROR: downloaded hey cannot execute on $os/$arch." >&2
        rm -f "$HEY"
        return 1
      fi
      ;;
    *)
      echo "ERROR: no prebuilt 'hey' binary for $os/$arch." >&2
      echo "       Install manually: 'brew install hey' or 'go install github.com/rakyll/hey@latest'" >&2
      return 1
      ;;
  esac
}

# Runs a CLI command, indents, and limits output
run_cli_head() {
  local lines="${1:-20}"
  shift
  local display_cmd="systemprompt $*"
  cmd "$display_cmd"
  local output
  output=$("$CLI" "$@" --profile "$PROFILE" 2>&1)
  local total
  total=$(echo "$output" | wc -l)
  echo "$output" | head -"$lines" | sed 's/^/  /'
  if [[ "$total" -gt "$lines" ]]; then
    echo "  ... ($((total - lines)) more lines)"
  fi
  echo ""
}
