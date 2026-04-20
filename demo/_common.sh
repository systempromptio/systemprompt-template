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
  TOKEN="${1:-}"
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
    Darwin/*)                  url="https://hey-release.s3.us-east-2.amazonaws.com/hey_darwin_amd64" ;;
    Linux/x86_64|Linux/amd64)  url="https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64" ;;
    *)
      echo "ERROR: no prebuilt 'hey' binary for $os/$arch." >&2
      echo "       Install manually: 'brew install hey' or 'go install github.com/rakyll/hey@latest'" >&2
      return 1
      ;;
  esac
  echo "  Installing hey from $url ..."
  if ! curl -fsSL "$url" -o "$HEY"; then
    echo "ERROR: failed to download hey from $url" >&2
    return 1
  fi
  chmod +x "$HEY"
  if ! "$HEY" --help >/dev/null 2>&1; then
    echo "ERROR: downloaded hey cannot execute on $os/$arch." >&2
    if [[ "$os/$arch" == "Darwin/arm64" ]]; then
      echo "       Apple Silicon needs Rosetta 2: 'softwareupdate --install-rosetta'" >&2
      echo "       or install a native build: 'brew install hey'" >&2
    fi
    rm -f "$HEY"
    return 1
  fi
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
