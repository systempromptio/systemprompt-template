#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
core="${CORE_REPO:-../systemprompt-core}"
ref_file=CORE_REF
[ ! -f bridge/CORE_REF ] || ref_file=bridge/CORE_REF
ref="$(tr -d '[:space:]' < "$ref_file")"
[ "$(git -C "$core" rev-parse HEAD)" = "$ref" ] || {
    echo "core checkout differs from $ref_file ($ref)" >&2
    exit 1
}
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$core/Cargo.toml" | head -1)"
names=" systemprompt $(awk '/^systemprompt.* =.*path =/ { print $1 }' "$core/Cargo.toml" | tr '\n' ' ') "
for manifest in Cargo.toml tests/Cargo.toml crates/tests/Cargo.toml; do
    [ -f "$manifest" ] || continue
    grep -q '^\[patch\.crates-io\]' "$manifest" || {
        echo "$manifest must override core in next builds" >&2
        exit 1
    }
done
for lock in Cargo.lock tests/Cargo.lock crates/tests/Cargo.lock bridge/Cargo.lock; do
    [ -f "$lock" ] || continue
    awk -v names="$names" -v expected="$version" -v lock="$lock" '
        function check() {
            if (name != "" && index(names, " " name " ") && (version != expected || source != "")) {
                print lock ": " name " must resolve from pinned core " expected ", got " version " " source > "/dev/stderr"
                failed = 1
            }
        }
        /^\[\[package\]\]/ { check(); name = ""; version = ""; source = ""; next }
        /^name = "/ { name = $0; sub(/^name = "/, "", name); sub(/"$/, "", name) }
        /^version = "/ { version = $0; sub(/^version = "/, "", version); sub(/"$/, "", version) }
        /^source = "/ { source = $0 }
        END { check(); exit failed }
    ' "$lock"
done
echo "core resolution: $ref ($version), all workspaces use the pinned checkout"
