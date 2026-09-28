#!/usr/bin/env bash
# Every systemprompt-* crate must resolve to ONE version across the three
# lockfiles (root, tests/, bridge/). bridge/ depends on the sibling core's
# bin/bridge by path, and tests/ is its own workspace, so a `cargo update -w`
# in the root alone leaves the others on the previous core: two copies of the
# same crate at different versions then surface as an unrelated compile error
# mid-release (0.51.0). Path and registry copies may coexist, but only at the
# same version.
set -euo pipefail

cd "$(dirname "$0")/.."

lockfiles=()
for lf in Cargo.lock tests/Cargo.lock bridge/Cargo.lock; do
    [ -f "$lf" ] && lockfiles+=("$lf")
done
[ "${#lockfiles[@]}" -gt 0 ] || { echo "check-core-crate-versions: no lockfiles found" >&2; exit 1; }

# One line per (crate, version, lockfile), parsed from [[package]] blocks.
rows="$(
    for lf in "${lockfiles[@]}"; do
        awk -v lf="$lf" '
            /^\[\[package\]\]/ { name = ""; version = ""; next }
            /^name = "/    { gsub(/^name = "|"$/, "");    name = $0; next }
            /^version = "/ { gsub(/^version = "|"$/, ""); version = $0;
                             if (name ~ /^systemprompt/) print name, version, lf; next }
        ' "$lf"
    done | sort -u
)"

drift="$(
    printf '%s\n' "$rows" \
        | awk '{ key = $1 SUBSEP $2; if (!(key in where)) versions[$1]++; where[key] = where[key] " " $3 }
               END { for (key in where) { split(key, kv, SUBSEP); if (versions[kv[1]] > 1) print kv[1], kv[2], where[key] } }' \
        | sort
)"

if [ -n "$drift" ]; then
    echo "check-core-crate-versions: systemprompt crates resolve to more than one version:" >&2
    printf '%s\n' "$drift" | awk '{ printf "  %-40s %-10s in%s\n", $1, $2, substr($0, length($1 $2) + 3) }' >&2
    echo "fix: cargo update -w for the root, tests/Cargo.toml and bridge/Cargo.toml manifests (see just core-bump)" >&2
    exit 1
fi

crates="$(printf '%s\n' "$rows" | awk '{ print $1 }' | sort -u | wc -l)"
echo "✓ check-core-crate-versions: $crates systemprompt crates at one version each across ${lockfiles[*]}"
