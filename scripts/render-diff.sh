#!/usr/bin/env bash
#
# render-diff.sh - SSR render-diff harness for the typed-template-context refactor.
#
# Captures whitespace-normalized HTML for every SSR route (public site + admin)
# into a named snapshot dir, and diffs two snapshots route-by-route. Use it to
# prove a refactor produces byte-identical rendered output.
#
#   scripts/render-diff.sh capture <name>
#   scripts/render-diff.sh compare <base> <new>
#
# Env overrides:
#   BASE_URL   server to hit           (default http://127.0.0.1:8099)
#   OUT_ROOT   snapshot root dir       (default <scratchpad>/render-diff)
#   ROUTES     route inventory file    (default scripts/render-diff-routes.txt)
#   TOKEN      admin bearer/cookie JWT  (default: mint via `systemprompt admin session login`)
#
# macOS + Linux safe: no `grep -P`, no `head -n -1`, no GNU-only sed features.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BASE_URL="${BASE_URL:-http://127.0.0.1:8099}"
ROUTES="${ROUTES:-$SCRIPT_DIR/render-diff-routes.txt}"
OUT_ROOT="${OUT_ROOT:-/tmp/claude-1000/-var-www-html-systemprompt-template/8be86b90-2aa3-460e-8732-35ca05d76d70/scratchpad/render-diff}"

die() { echo "error: $*" >&2; exit 2; }

# --- admin token ------------------------------------------------------------
# The admin SSR routes read the JWT from the `access_token` cookie (or a Bearer
# header). We mint one with the CLI unless TOKEN is already exported. The token
# must be signed with the key the target server serves at /.well-known/jwks.json;
# a clean `just start` of the local profile satisfies this.
ADMIN_TOKEN=""
ensure_token() {
  [ -n "$ADMIN_TOKEN" ] && return 0
  if [ -n "${TOKEN:-}" ]; then
    ADMIN_TOKEN="$TOKEN"
    return 0
  fi
  local bin="$REPO_ROOT/target/debug/systemprompt"
  [ -x "$bin" ] || bin="systemprompt"
  ADMIN_TOKEN="$("$bin" admin session login --token-only 2>/dev/null | tail -1)"
  [ -n "$ADMIN_TOKEN" ] || die "could not mint admin token (is the local profile server up?)"
}

# --- normalization ----------------------------------------------------------
# Strip volatile content so two captures of an unchanged server are byte-equal:
#   * put each tag on its own line and trim, so structural diffs are readable
#   * blank out CSP nonces, CSRF tokens, request/trace/instance ids that rotate
#   * blank out wall-clock timestamps and relative "N ago" / "Nh Nm" durations
# Keep it deterministic: same input -> same output on every run and both OSes.
normalize() {
  sed -E \
    -e 's/>[[:space:]]*</>\
</g' \
  | sed -E \
    -e 's/nonce="[^"]*"/nonce="NONCE"/g' \
    -e 's/(name="[^"]*csrf[^"]*"[^>]*value=)"[^"]*"/\1"CSRF"/gI' \
    -e 's/(data-csrf(-token)?=)"[^"]*"/\1"CSRF"/gI' \
    -e 's/(csrf[_-]?token"?[[:space:]]*[:=][[:space:]]*)"[^"]*"/\1"CSRF"/gI' \
    -e 's/x-served-by[^"[:space:]]*/x-served-by-INSTANCE/gI' \
    -e 's/instance-[0-9a-f]{16,}/instance-INSTANCE/g' \
    -e 's/[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]+)?(Z|[+-][0-9]{2}:?[0-9]{2})?/TIMESTAMP/g' \
    -e 's/[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}(:[0-9]{2})?/TIMESTAMP/g' \
    -e 's/[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}%3A[0-9]{2}%3A[0-9]{2}[^"&]*/TIMESTAMP/gI' \
    -e 's/[0-9]+[[:space:]]*(second|minute|hour|day|week|month|year)s?[[:space:]]+ago/DURATION ago/gI' \
    -e 's/[0-9]+h[[:space:]]+[0-9]+m([[:space:]]+ago)?/DURATION/gI' \
    -e 's/[0-9]+[smhd][[:space:]]+ago/DURATION ago/gI' \
    -e 's/in[[:space:]]+[0-9]+[[:space:]]*(second|minute|hour|day)s?/in DURATION/gI' \
    -e 's/^[[:space:]]+//' \
  | sed -E '/^$/d' \
  | sed -E '/CLI Session - /d'
  # The last rule drops rows for ephemeral CLI-session contexts: the CLI mints
  # a "CLI Session - <profile>" user_contexts row on every stale-session
  # invocation with no GC, so row membership churns between captures. Tracked
  # as a core fix (kind column / GC) in systemprompt-core
  # internal/tickets/user-contexts-ephemeral-cli-sessions.md.
}

fetch_one() {
  # $1 = route (may start with '@' for admin)
  local raw="$1" path curl_args=()
  case "$raw" in
    @*) path="${raw#@}"; ensure_token; curl_args=(--cookie "access_token=$ADMIN_TOKEN") ;;
    *)  path="$raw" ;;
  esac
  curl -sS -m 30 "${curl_args[@]}" "$BASE_URL$path"
}

# Map a route to a filesystem-safe snapshot filename.
route_slug() {
  local s
  s="$(printf '%s' "$1" | sed -E 's/^@//; s/[^A-Za-z0-9._-]/_/g; s/^_+//; s/_+$//')"
  [ -n "$s" ] || s="root"
  printf '%s' "$s"
}

read_routes() {
  # Strip comments/blank lines; emit one route per line.
  sed -E 's/#.*$//; s/[[:space:]]+$//' "$ROUTES" | sed -E '/^[[:space:]]*$/d'
}

cmd_capture() {
  local name="${1:-}"
  [ -n "$name" ] || die "usage: render-diff.sh capture <name>"
  [ -f "$ROUTES" ] || die "routes file not found: $ROUTES"

  local dir="$OUT_ROOT/$name"
  rm -rf "$dir"
  mkdir -p "$dir"

  local count=0
  while IFS= read -r route; do
    [ -n "$route" ] || continue
    local slug; slug="$(route_slug "$route")"
    local status
    status="$(fetch_one "$route" | normalize > "$dir/$slug.html"; echo done)" || true
    # record the http status alongside for triage
    local http
    case "$route" in
      @*) ensure_token; http="$(curl -sS -m 30 -o /dev/null -w '%{http_code}' --cookie "access_token=$ADMIN_TOKEN" "$BASE_URL${route#@}")" ;;
      *)  http="$(curl -sS -m 30 -o /dev/null -w '%{http_code}' "$BASE_URL$route")" ;;
    esac
    printf '%s\t%s\n' "$http" "$route" >> "$dir/_status.tsv"
    count=$((count + 1))
  done < <(read_routes)

  echo "captured $count routes -> $dir"
}

cmd_compare() {
  local base="${1:-}" new="${2:-}"
  [ -n "$base" ] && [ -n "$new" ] || die "usage: render-diff.sh compare <base> <new>"
  local bdir="$OUT_ROOT/$base" ndir="$OUT_ROOT/$new"
  [ -d "$bdir" ] || die "no such capture: $bdir"
  [ -d "$ndir" ] || die "no such capture: $ndir"

  local diffs=0 checked=0
  # Union of html files across both captures.
  local files
  files="$( (cd "$bdir" && ls *.html 2>/dev/null; cd "$ndir" && ls *.html 2>/dev/null) | sort -u )"
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    checked=$((checked + 1))
    if [ ! -f "$bdir/$f" ]; then
      echo "ONLY-IN-NEW  $f"; diffs=$((diffs + 1)); continue
    fi
    if [ ! -f "$ndir/$f" ]; then
      echo "ONLY-IN-BASE $f"; diffs=$((diffs + 1)); continue
    fi
    if ! diff -q "$bdir/$f" "$ndir/$f" >/dev/null 2>&1; then
      echo "DIFF         $f"
      diff -u "$bdir/$f" "$ndir/$f" | sed -E 's/^/    /' | head -40
      diffs=$((diffs + 1))
    fi
  done < <(printf '%s\n' "$files")

  echo "----"
  echo "compared $checked routes: $diffs differ"
  [ "$diffs" -eq 0 ] || return 1
}

main() {
  local sub="${1:-}"; shift || true
  case "$sub" in
    capture) cmd_capture "$@" ;;
    compare) cmd_compare "$@" ;;
    *) die "usage: render-diff.sh {capture <name>|compare <base> <new>}" ;;
  esac
}

main "$@"
