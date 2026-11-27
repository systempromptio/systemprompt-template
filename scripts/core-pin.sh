#!/usr/bin/env bash
# Pins systemprompt-core by git rev. bridge/CORE_REF holds the one rev; every
# core crate the root, tests/ and extensions/mobile/core manifests name (each
# is a separate workspace) is a git dependency on that rev, so core's own
# inter-crate dependencies resolve inside the same git source and each
# lockfile carries exactly one copy of every systemprompt-* crate.
#   scripts/core-pin.sh <rev>    write CORE_REF and every entry, re-resolve every lock
#   scripts/core-pin.sh --check  fail on another rev, a registry or duplicate core crate, or an override
set -euo pipefail
cd "$(dirname "$0")/.."
url=https://github.com/systempromptio/systemprompt-core
ref_file=bridge/CORE_REF
manifests=(Cargo.toml tests/Cargo.toml extensions/mobile/core/Cargo.toml)
locks=(Cargo.lock tests/Cargo.lock extensions/mobile/core/Cargo.lock)

core_entries() {
    grep -E '^systemprompt[a-z-]* = \{' "$1" | grep -vE 'path = |workspace = true' || true
}

lock_sources() {
    awk '/^\[\[package\]\]/ { name = ""; next }
         /^name = "/ { name = $3; gsub(/"/, "", name); next }
         /^source = "/ && name ~ /^systemprompt/ { print name, $3 }' "$1"
}

check() {
    local ref fail=0 m l bad
    ref="$(tr -d '[:space:]' < "$ref_file")"
    [[ "$ref" =~ ^[0-9a-f]{40}$ ]] || { echo "core-pin: $ref_file must hold one 40-char core SHA, got '$ref'" >&2; exit 1; }
    for m in "${manifests[@]}"; do
        if grep -qE '^\[patch[.]' "$m"; then
            echo "core-pin: $m carries a [patch] table (a local core override); run scripts/core-patch.sh off" >&2
            fail=1
        fi
        [ -n "$(core_entries "$m")" ] || { echo "core-pin: $m names no core crate" >&2; fail=1; continue; }
        if core_entries "$m" | grep -vF "git = \"$url\", rev = \"$ref\""; then
            echo "core-pin: the entries above in $m are not git dependencies on $url at $ref_file ($ref)" >&2
            fail=1
        fi
    done
    for l in "${locks[@]}"; do
        bad="$(lock_sources "$l" | grep -vF "\"git+$url?rev=$ref#$ref\"" || true)"
        [ -z "$bad" ] || { printf '%s\n' "$bad" >&2; echo "core-pin: $l resolves the core crates above from another source; run scripts/core-pin.sh $ref" >&2; fail=1; }
        bad="$(lock_sources "$l" | awk '{ print $1 }' | sort | uniq -d)"
        [ -z "$bad" ] || { printf '%s\n' "$bad" >&2; echo "core-pin: $l holds more than one copy of the core crates above" >&2; fail=1; }
        [ -n "$(lock_sources "$l")" ] || { echo "core-pin: $l resolves no core crate from $ref" >&2; fail=1; }
    done
    [ "$fail" -eq 0 ] || exit 1
    echo "core-pin: every workspace (${manifests[*]}) resolves one copy of each core crate from $url at $ref"
}

case "${1:?usage: scripts/core-pin.sh <rev> | --check}" in
    --check) check ;;
    *)
        [ "$(bash scripts/core-patch.sh status)" = pin ] || { echo "core-pin: a local core override is on; run scripts/core-patch.sh off first" >&2; exit 1; }
        ref="$1"
        if ! [[ "$ref" =~ ^[0-9a-f]{40}$ ]]; then
            ref="$(git ls-remote "$url" | awk -v r="$ref" '$1 ~ "^"r || $2 == "refs/heads/"r || $2 == "refs/tags/"r { print $1; exit }')"
            [ -n "$ref" ] || { echo "core-pin: $1 names no commit, branch or tag of $url; pass the full SHA" >&2; exit 1; }
        fi
        printf '%s\n' "$ref" > "$ref_file"
        for m in "${manifests[@]}"; do
            # Why: the address is delimited with "#" because the URL holds
            # slashes, and the edit goes through a copy because `sed -i` takes
            # different arguments on macOS and Linux.
            sed -E "\\#^systemprompt[a-z-]* = \{.*git = \"$(printf '%s' "$url" | sed 's|[.]|[.]|g')\"# s|rev = \"[0-9a-f]{40}\"|rev = \"$ref\"|" "$m" > "$m.pin"
            mv "$m.pin" "$m"
        done
        for m in "${manifests[@]}"; do
            cargo metadata --manifest-path "$m" --format-version 1 >/dev/null
        done
        check
        ;;
esac
