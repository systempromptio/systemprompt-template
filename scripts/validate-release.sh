#!/usr/bin/env bash
# Ensure release metadata names the checked-out, published-core source.
set -euo pipefail
cd "$(dirname "$0")/.."
tag="${1:?usage: validate-release.sh vX.Y.Z}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Invalid release tag: $tag" >&2; exit 1; }
test "$(git rev-parse HEAD)" = "$(git rev-parse "$tag^{commit}")"
git merge-base --is-ancestor HEAD origin/main
bash scripts/sync-release-version.sh "${tag#v}" --check
if grep -Eq '^\[patch\.crates-io\]|^[^#]*path.*(\.\./)+systemprompt-core' Cargo.toml tests/Cargo.toml; then
    echo "Release must resolve core from crates.io" >&2; exit 1
fi
cargo metadata --locked --no-deps --format-version 1 >/dev/null
cargo metadata --manifest-path tests/Cargo.toml --locked --no-deps --format-version 1 >/dev/null
