#!/usr/bin/env bash
#
# Single-flight coordinator for the expensive workspace recipes (build, clippy,
# tests, gates).
#
# Several agents working the same clone each run `just build` to check their
# work. Cargo's own lock makes those runs serialize, so the Nth agent waits for
# N-1 full builds it did not need: every one of them compiles the same source
# tree and produces the same answer.
#
# This wrapper collapses that into one run, and records the answer where the
# other agents can read it:
#
#   leader   - first caller for a given (recipe, flags, source fingerprint)
#              takes the lock and runs the real command, teeing to a log.
#   follower - a later caller with the SAME fingerprint attaches to the
#              leader's log and exits with the leader's status. No second run.
#   waiter   - a later caller with a DIFFERENT fingerprint (someone edited a
#              file mid-flight) waits for the lock, then becomes leader.
#   hit      - the fingerprint already has a recorded success, so there is
#              nothing to do. Returns immediately.
#
# The ledger lives in ./.build (gitignored, not inside target/, so it survives
# `cargo clean` and is safe to read while a build is writing target/):
#
#   .build/runs.jsonl          append-only history, newest last
#   .build/latest/<recipe>.json  most recent outcome per recipe
#   .build/logs/<recipe>-<key>.log
#   .build/lock/               the in-flight run (pid, key, recipe, log)
#
# Read it before starting your own run: `just build-status`.
#
# Usage: build-coordinator.sh run <recipe> <flags> -- <command...>
#        build-coordinator.sh status [recipe]
#
# Env: BUILD_FORCE=1     ignore the success cache (still single-flight)
#      BUILD_NO_COORD=1  bypass entirely and exec the command
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# COORD_STATE_DIR points the ledger and the lock somewhere else. Used by the
# self-test so exercising the coordinator cannot disturb a real build in flight.
STATE_DIR="${COORD_STATE_DIR:-$ROOT/.build}"
LOCK="$STATE_DIR/lock"
RUNS="$STATE_DIR/runs.jsonl"
MAX_RUNS=200

# Everything the compiler reads. `services/` is in the list because the
# public-site partials are include_str!-compiled into the binary.
FINGERPRINT_PATHS=(
    Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml
    src extensions migrations tests services scripts .sqlx bridge
)

sha() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum | cut -d' ' -f1
    else shasum -a 256 | cut -d' ' -f1; fi
}

now() { date -u '+%Y-%m-%dT%H:%M:%SZ'; }

# Content hash of a checkout: committed state, uncommitted diff, and the
# contents of untracked files. Two trees with the same value compile to the
# same artifacts.
tree_fingerprint() {
    local dir="$1"; shift
    {
        git -C "$dir" rev-parse HEAD 2>/dev/null || echo no-git
        git -C "$dir" diff HEAD -- "$@" 2>/dev/null || true
        git -C "$dir" ls-files --others --exclude-standard -- "$@" 2>/dev/null |
            while IFS= read -r f; do
                printf '%s %s\n' "$f" "$(git -C "$dir" hash-object "$dir/$f" 2>/dev/null || echo gone)"
            done
    } | sha
}

# When [patch.crates-io] is live, the sibling core checkout is a build input
# and must be part of the fingerprint.
core_fingerprint() {
    local core="$ROOT/../systemprompt-core"
    if grep -qE '^\[patch\.crates-io\]' "$ROOT/Cargo.toml" && [ -d "$core" ]; then
        tree_fingerprint "$core" .
    else
        echo unpatched
    fi
}

# The tree hash answers "is this the same source?" and is shared by every
# recipe. The run key adds the recipe and its flags, because `build --release`
# passing says nothing about `clippy` passing on the same tree.
compute_tree() {
    # A caller reporting on several things at once (server-state.sh) pins one
    # value so its output stays internally consistent while other agents keep
    # editing the tree underneath it.
    if [ -n "${COORD_CURRENT_KEY:-}" ]; then printf '%s\n' "$COORD_CURRENT_KEY"; return; fi
    printf '%s\n%s\n' \
        "$(tree_fingerprint "$ROOT" "${FINGERPRINT_PATHS[@]}")" \
        "$(core_fingerprint)" | sha
}

compute_key() {
    printf '%s\n%s\n%s\n' "$1" "$2" "$(compute_tree)" | sha
}

lock_alive() {
    local pid
    pid="$(cat "$LOCK/pid" 2>/dev/null || echo)"
    [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null
}

json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

# One line per finished run, plus a per-recipe pointer to the newest.
record_run() {
    local recipe="$1" flags="$2" key="$3" status="$4" started="$5" log="$6" outcome="$7"
    local line
    line=$(printf '{"recipe":"%s","flags":"%s","key":"%s","tree":"%s","status":%s,"outcome":"%s","started":"%s","finished":"%s","log":"%s","pid":%s}' \
        "$(json_escape "$recipe")" "$(json_escape "$flags")" "$key" "$TREE" "$status" "$outcome" \
        "$started" "$(now)" "$(json_escape "$log")" "$$")
    mkdir -p "$STATE_DIR/latest"
    printf '%s\n' "$line" >> "$RUNS"
    printf '%s\n' "$line" > "$STATE_DIR/latest/$recipe.json"
    if [ "$(wc -l < "$RUNS")" -gt "$MAX_RUNS" ]; then
        tail -n "$MAX_RUNS" "$RUNS" > "$RUNS.tmp" && mv "$RUNS.tmp" "$RUNS"
    fi
}

# Provenance: which source fingerprint produced the binary now on disk. This is
# what lets `just start` tell a running server apart from a stale one - the
# server's own /proc exe hashes to one of these lines, or to none.
record_binaries() {
    local bin bsha
    for bin in target/debug/systemprompt target/release/systemprompt; do
        [ -x "$ROOT/$bin" ] || continue
        bsha="$(sha < "$ROOT/$bin")"
        printf '{"path":"%s","binary_sha":"%s","tree":"%s","built":"%s"}\n' \
            "$bin" "$bsha" "$TREE" "$(now)" >> "$STATE_DIR/binaries.jsonl"
    done
    if [ -f "$STATE_DIR/binaries.jsonl" ] && [ "$(wc -l < "$STATE_DIR/binaries.jsonl")" -gt "$MAX_RUNS" ]; then
        tail -n "$MAX_RUNS" "$STATE_DIR/binaries.jsonl" > "$STATE_DIR/binaries.tmp"
        mv "$STATE_DIR/binaries.tmp" "$STATE_DIR/binaries.jsonl"
    fi
}

# Replay a leader's log to our stdout, following it until the leader records a
# status, which lands in FOLLOW_STATUS. Written by hand because macOS `tail` has
# no --pid. The status must not go to stdout: stdout is the leader's output,
# which the caller is relaying verbatim to its own agent.
FOLLOW_STATUS=1
follow_log() {
    local log="$1" status_file="$2" off=0 size
    while :; do
        if [ -f "$log" ]; then
            size="$(wc -c < "$log" | tr -d ' ')"
            if [ "$size" -gt "$off" ]; then
                tail -c "+$((off + 1))" "$log"
                off="$size"
            fi
        fi
        [ -f "$status_file" ] && break
        sleep 0.4
    done
    if [ -f "$log" ]; then
        size="$(wc -c < "$log" | tr -d ' ')"
        [ "$size" -gt "$off" ] && tail -c "+$((off + 1))" "$log"
    fi
    FOLLOW_STATUS="$(cat "$status_file" 2>/dev/null || echo 1)"
}

cmd_status() {
    local want="${1:-}"
    if [ -d "$LOCK" ] && lock_alive "$LOCK"; then
        echo "IN FLIGHT  $(cat "$LOCK/recipe" 2>/dev/null || echo '?')  pid=$(cat "$LOCK/pid")  since=$(cat "$LOCK/started" 2>/dev/null || echo '?')"
        echo "  log: $(cat "$LOCK/log-path" 2>/dev/null || echo "$LOCK/log")"
        echo "  --- tail ---"
        tail -n 15 "$LOCK/log" 2>/dev/null | sed 's/^/  /' || true
        echo
    else
        echo "IN FLIGHT  none"
        echo
    fi
    local current
    current="$(compute_tree)"
    echo "current source fingerprint: $(cut -c1-12 <<<"$current")"
    echo
    if [ ! -d "$STATE_DIR/latest" ]; then
        echo "no recorded runs yet"
        return 0
    fi
    printf '%-16s %-8s %-14s %-22s %s\n' RECIPE RESULT FINGERPRINT FINISHED FRESH
    local f recipe status key finished fresh
    for f in "$STATE_DIR/latest"/*.json; do
        [ -e "$f" ] || continue
        recipe="$(basename "$f" .json)"
        [ -n "$want" ] && [ "$want" != "$recipe" ] && continue
        status="$(sed -n 's/.*"status":\([0-9-]*\).*/\1/p' "$f")"
        key="$(sed -n 's/.*"tree":"\([^"]*\)".*/\1/p' "$f")"
        finished="$(sed -n 's/.*"finished":"\([^"]*\)".*/\1/p' "$f")"
        # A recorded run only tells you about the tree it ran against. If the
        # fingerprint moved, the result is stale and says nothing about now.
        if [ -z "$key" ]; then fresh="unknown (record predates fingerprinting)"
        elif [ "$key" = "$current" ]; then fresh="yes (source unchanged)"
        else fresh="no (source changed since)"; fi
        printf '%-16s %-8s %-14s %-22s %s\n' \
            "$recipe" "$([ "$status" = 0 ] && echo PASS || echo "FAIL($status)")" \
            "$(cut -c1-12 <<<"$key")" "$finished" "$fresh"
    done
    echo
    echo "logs: $STATE_DIR/logs   history: $RUNS"
}

cmd_run() {
    local recipe="$1" flags="$2"; shift 2
    [ "${1:-}" = "--" ] && shift

    if [ -n "${BUILD_NO_COORD:-}" ]; then exec "$@"; fi

    mkdir -p "$STATE_DIR/logs" "$STATE_DIR/latest"
    # One tree hash for the whole run, so the record, the lock, and the success
    # marker all describe the same source even as other agents keep editing.
    TREE="$(compute_tree)"; export COORD_CURRENT_KEY="$TREE"
    local key; key="$(compute_key "$recipe" "$flags")"

    # Reentrancy: coordinated recipes depend on each other (`clippy` runs
    # `lint-gates`). A nested call must not queue behind the lock its own
    # parent is holding, so it runs inline and only records the outcome.
    if [ -n "${BUILD_COORD_ACTIVE:-}" ]; then
        local nested_started; nested_started="$(now)"
        set +e; "$@"; local nrc=$?; set -e
        record_run "$recipe" "$flags" "$key" "$nrc" "$nested_started" "" nested
        [ "$nrc" -eq 0 ] && { rm -f "$STATE_DIR/success-$recipe-"*; : > "$STATE_DIR/success-$recipe-$key"; }
        return "$nrc"
    fi
    local success_file="$STATE_DIR/success-$recipe-$key"
    local log="$STATE_DIR/logs/$recipe-$(cut -c1-12 <<<"$key").log"

    if [ -z "${BUILD_FORCE:-}" ] && [ -f "$success_file" ]; then
        echo "[coord] $recipe $flags: already green for this source tree ($(cut -c1-12 <<<"$key"))."
        echo "[coord] log: $log   details: just build-status   recompile anyway: BUILD_FORCE=1"
        record_run "$recipe" "$flags" "$key" 0 "$(now)" "$log" cache-hit
        return 0
    fi

    while :; do
        if mkdir "$LOCK" 2>/dev/null; then break; fi
        if ! lock_alive "$LOCK"; then
            # mv is atomic, so concurrent waiters cannot both clear the stale
            # lock and then delete a successor's freshly-created lock dir; the
            # loser's mv fails and it simply retries the mkdir.
            if mv "$LOCK" "$LOCK.stale.$$" 2>/dev/null; then
                echo "[coord] clearing stale lock from a dead process" >&2
                rm -rf "$LOCK.stale.$$"
            fi
            continue
        fi
        local owner_key owner_recipe
        owner_key="$(cat "$LOCK/key" 2>/dev/null || echo)"
        owner_recipe="$(cat "$LOCK/recipe" 2>/dev/null || echo run)"
        if [ "$owner_key" = "$key" ] && [ "$owner_recipe" = "$recipe $flags" ]; then
            echo "[coord] identical '$recipe $flags' already running (pid $(cat "$LOCK/pid")); attaching to its output instead of starting a second one."
            follow_log "$LOCK/log" "$LOCK/status"
            echo "[coord] leader finished with status $FOLLOW_STATUS"
            record_run "$recipe" "$flags" "$key" "$FOLLOW_STATUS" "$(now)" "$log" attached
            return "$FOLLOW_STATUS"
        fi
        echo "[coord] '$owner_recipe' holds the lock (pid $(cat "$LOCK/pid" 2>/dev/null || echo '?')); waiting..."
        while [ -d "$LOCK" ] && lock_alive "$LOCK"; do sleep 1; done
    done

    # We are the leader.
    trap 'rm -rf "$LOCK"' EXIT INT TERM
    local started; started="$(now)"
    echo $$ > "$LOCK/pid"
    echo "$key" > "$LOCK/key"
    echo "$recipe $flags" > "$LOCK/recipe"
    echo "$started" > "$LOCK/started"
    echo "$log" > "$LOCK/log-path"
    : > "$LOCK/log"

    echo "[coord] leading '$recipe $flags' (fingerprint $(cut -c1-12 <<<"$key"))"
    set +e
    BUILD_COORD_ACTIVE=1 "$@" 2>&1 | tee "$LOCK/log"
    local rc=${PIPESTATUS[0]}
    set -e
    echo "$rc" > "$LOCK/status"
    cp "$LOCK/log" "$log" 2>/dev/null || true

    if [ "$rc" -eq 0 ]; then
        rm -f "$STATE_DIR/success-$recipe-"*
        : > "$success_file"
        [ "$recipe" = build ] && record_binaries
    fi
    record_run "$recipe" "$flags" "$key" "$rc" "$started" "$log" leader
    # Give attached followers a beat to drain the log before the lock vanishes.
    sleep 1
    return "$rc"
}

case "${1:-}" in
    status) shift; cmd_status "$@" ;;
    run)    shift; cmd_run "$@" ;;
    key)    shift; compute_tree ;;
    *)      echo "usage: $0 run <recipe> <flags> -- <command...> | status [recipe] | key" >&2; exit 2 ;;
esac
