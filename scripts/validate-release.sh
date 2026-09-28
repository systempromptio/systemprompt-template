#!/usr/bin/env bash
# Ensure release metadata names the checked-out, published-core source.
set -euo pipefail
cd "$(dirname "$0")/.."
tag="${1:?usage: validate-release.sh vX.Y.Z}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Invalid release tag: $tag" >&2; exit 1; }
test "$(git rev-parse HEAD)" = "$(git rev-parse "$tag^{commit}")"
git merge-base --is-ancestor HEAD origin/main
bash scripts/sync-release-version.sh "${tag#v}" --check
# A live [patch.crates-io] table, or a ../systemprompt-core path anywhere
# except the inert [workspace.metadata.unreleased-core-patch] table (the
# override set kept for the next unreleased-core cycle), resolves core from a
# local checkout rather than crates.io.
if awk '
    /^\[/ { inert = ($0 ~ /^\[workspace\.metadata\./) }
    /^\[patch\.crates-io\]/ { live = 1 }
    !inert && /^[^#]*path.*(\.\.\/)+systemprompt-core/ { live = 1 }
    END { exit live ? 0 : 1 }
' Cargo.toml tests/Cargo.toml; then
    echo "Release must resolve core from crates.io" >&2; exit 1
fi
cargo metadata --locked --no-deps --format-version 1 >/dev/null
cargo metadata --manifest-path tests/Cargo.toml --locked --no-deps --format-version 1 >/dev/null
