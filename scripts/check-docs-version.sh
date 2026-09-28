#!/usr/bin/env bash
# Gate: documentation cannot name a release the crates are not on.
#
# Two halves. Hosted pages under services/content/ are rendered by the running
# binary, so they must never carry a literal 0.x.y (write "the current
# release", or link the releases page). Operator docs read on GitHub — every
# channel recipe under docs/install/ and the deploy/*/ runbooks — carry the
# real number, and every one of them must equal the workspace version in
# Cargo.toml, which scripts/sync-release-version.sh rewrites on every bump.
#
# One exemption: a sentence about a specific past release (for example "the
# 0.44.0 assets are unsigned") is a historical fact, and substituting the
# current version would make it false. Such a line must end with the marker
# `<!-- pinned-release -->`, which renders invisibly and makes the exemption
# review-visible. Nothing else is exempt.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)
[ -n "$version" ] || { echo "check-docs-version: cannot read workspace version from Cargo.toml"; exit 1; }
fail=0

hosted=$(grep -rnE '(^|[^0-9.])0\.[0-9]+\.[0-9]+([^0-9.]|$)' services/content/ --include='*.md' \
    | grep -v '<!-- pinned-release -->$' || true)
if [ -n "$hosted" ]; then
    echo "hosted documentation must not name a literal version:"
    echo "$hosted"
    fail=1
fi

for doc in docs/install/*.md deploy/*/*.md; do
    [ -f "$doc" ] || continue
    other=$(grep -nE '(^|[^0-9.])0\.[0-9]+\.[0-9]+([^0-9.]|$)' "$doc" | grep -v "$version" \
        | grep -v '<!-- pinned-release -->$' || true)
    if [ -n "$other" ]; then
        echo "$doc names a version other than the workspace version $version (run scripts/sync-release-version.sh $version):"
        echo "$other"
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then exit 1; fi
echo "docs version OK: hosted pages tokenised, operator docs on $version"
