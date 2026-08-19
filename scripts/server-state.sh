#!/usr/bin/env bash
#
# What is the local server doing right now, and is it built from current source?
#
# Several agents share this clone. Without this, each one runs `just start`,
# finds a port already bound or silently restarts someone else's server, and
# nobody can tell whether the process answering on :8080 contains their change.
#
# `report` answers three questions:
#   1. is a server running, and since when
#   2. which binary is it running, and which source fingerprint built that
#      binary (via .build/binaries.jsonl, written by the build coordinator)
#   3. does that fingerprint still match the working tree
#
# This only ever reports. It does not start, stop, or restart anything -
# `just start` is untouched and behaves exactly as it always has.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="$ROOT/.build"
BINARIES="$STATE_DIR/binaries.jsonl"

sha() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum | cut -d' ' -f1
    else shasum -a 256 | cut -d' ' -f1; fi
}
short() { cut -c1-12; }

# Resolve the executable a pid is actually running. /proc on Linux; `ps` gives
# only the argv path on macOS, which is good enough to hash. `readlink` marks a
# binary that was rebuilt out from under a running process as "(deleted)" -
# keep that, it is the strongest staleness signal there is.
pid_exe() {
    local pid="$1" exe=""
    [ -r "/proc/$pid/exe" ] && exe="$(readlink "/proc/$pid/exe" 2>/dev/null || true)"
    if [ -z "$exe" ]; then
        exe="$(ps -o args= -p "$pid" 2>/dev/null | awk '{print $1}')"
        case "$exe" in ./*) exe="$ROOT/${exe#./}" ;; esac
    fi
    printf '%s' "$exe"
}

# Only real server processes. Matching the command line alone also catches the
# shell wrappers that spawned them, which are not servers.
server_pids() {
    local pid exe
    for pid in $(pgrep -f 'systemprompt (infra services (start|serve|restart)|serve)' 2>/dev/null || true); do
        exe="$(pid_exe "$pid")"
        case "$(basename "${exe% (deleted)}")" in
            systemprompt*) echo "$pid" ;;
        esac
    done
}

# The build fingerprint recorded for this exact binary content, or "" if this
# binary was never produced by a coordinated `just build`.
key_for_binary() {
    local bsha="$1"
    [ -f "$BINARIES" ] || return 0
    { grep -F "\"binary_sha\":\"$bsha\"" "$BINARIES" 2>/dev/null || true; } | tail -1 |
        sed -n 's/.*"tree":"\([^"]*\)".*/\1/p'
}

# Computed once and exported: other agents edit this tree continuously, and a
# report that hashed the source twice would compare its own two halves against
# different trees.
current_key() {
    if [ -z "${COORD_CURRENT_KEY:-}" ]; then
        COORD_CURRENT_KEY="$("$ROOT/scripts/build-coordinator.sh" key build "")"
        export COORD_CURRENT_KEY
    fi
    printf '%s' "$COORD_CURRENT_KEY"
}

# Is any source file newer than the binary? This works with no ledger history
# at all, which matters because the ledger only ever learns about builds run
# through the coordinator - a binary from `cargo build`, from a build that
# predates the coordinator, or from a checkout is still a perfectly good binary.
SOURCE_PATHS=(Cargo.toml Cargo.lock rust-toolchain.toml src extensions migrations services)
source_newer_than() {
    local bin="$1" p hit
    for p in "${SOURCE_PATHS[@]}"; do
        [ -e "$ROOT/$p" ] || continue
        hit="$(find "$ROOT/$p" -name target -prune -o -type f -newer "$bin" -print -quit 2>/dev/null)"
        [ -n "$hit" ] && return 0
    done
    return 1
}

# Sets FRESHNESS to one of:
#   current  - the coordinator built this exact binary from the current tree
#   fresh    - no ledger entry, but no source file is newer than it either
#   stale    - source has moved on since this binary was produced
#   replaced - the file this running process was executed from is gone
#   missing  - no such binary
FRESHNESS=missing
BUILT_KEY=""
classify_binary() {
    local path="$1" bsha built_key
    BUILT_KEY=""
    # A running process whose on-disk binary was replaced is stale by
    # definition: the file it was executed from no longer exists.
    case "$path" in *" (deleted)") FRESHNESS=replaced; return ;; esac
    if [ ! -x "$path" ]; then FRESHNESS=missing; return; fi
    bsha="$(sha < "$path")"
    built_key="$(key_for_binary "$bsha")"
    BUILT_KEY="$built_key"
    if [ -n "$built_key" ]; then
        # Exact provenance known.
        [ "$built_key" = "$(current_key)" ] && FRESHNESS=current || FRESHNESS=stale
        return
    fi
    # No ledger entry. Fall back to mtime, which still answers the only
    # question that matters: could this binary contain the current source?
    source_newer_than "$path" && FRESHNESS=stale || FRESHNESS=fresh
}

freshness_note() {
    case "$1" in
        current)  echo "built from the current source tree" ;;
        fresh)    echo "newer than every source file (no ledger entry - built outside the coordinator)" ;;
        stale)    echo "STALE - source has changed since this binary was built" ;;
        replaced) echo "STALE - the binary this process runs has been rebuilt and replaced on disk" ;;
        missing)  echo "not built" ;;
    esac
}

cmd_report() {
    local pids; pids="$(server_pids)"
    echo "── server ─────────────────────────────────────────────"
    if [ -z "$pids" ]; then
        echo "  not running"
    else
        local pid exe
        for pid in $pids; do
            exe="$(pid_exe "$pid")"
            classify_binary "$exe"
            echo "  pid $pid  up $(ps -o etime= -p "$pid" 2>/dev/null | tr -d ' ')"
            echo "    binary: $exe"
            echo "    build:  $(freshness_note "$FRESHNESS")${BUILT_KEY:+ (fingerprint $(short <<<"$BUILT_KEY"))}"
        done
    fi
    echo
    echo "── binary on disk ─────────────────────────────────────"
    local bin found=0
    for bin in target/debug/systemprompt target/release/systemprompt; do
        [ -e "$ROOT/$bin" ] || continue
        found=1
        classify_binary "$ROOT/$bin"
        echo "  $bin  $(freshness_note "$FRESHNESS")"
    done
    # Absent binaries are worth naming: mid-build there is a window where the
    # linker has removed the old one and not yet written the new one.
    [ "$found" -eq 0 ] && echo "  none present$([ -d "$STATE_DIR/lock" ] && echo " (a build is in flight - the linker has not written it yet)")"
    echo
    echo "── source ─────────────────────────────────────────────"
    echo "  current fingerprint: $(short <<<"$(current_key)")"
    echo
    "$ROOT/scripts/build-coordinator.sh" status
}

case "${1:-report}" in
    report) cmd_report ;;
    *) echo "usage: $0 [report]" >&2; exit 2 ;;
esac
