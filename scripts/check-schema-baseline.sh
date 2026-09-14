#!/usr/bin/env bash
# Gate: tests/fixtures/schema/release-baseline.sql records the schema the last
# release ships, and its header must name the workspace version.
#
# The upgrade test (tests/integration/schema-upgrade) restores that fixture
# into an empty database, runs the current installer over it, and diffs the
# result against a fresh install. That only proves anything if the fixture
# really is the previous release's schema. During a `next` cycle the
# workspace version is still the last release's, so this passes and the test
# upgrades *from* that release; `just core-bump X.Y.Z` bumps the version and
# turns this gate red until `just schema-baseline` re-records the fixture for
# X.Y.Z — which is exactly the moment it has to become X.Y.Z, before
# `just release`.
#
# Why a recorded fixture and not a download or a worktree build: the 0.52
# preview crash-looped on production (2026-09-14) on a declarative schema that
# installed cleanly on every fresh database and every local clone; the only
# database shaped like a deployed install was production. A tracked dump of
# the release is deterministic, offline, and identical in CI and locally.
set -euo pipefail

cd "$(dirname "$0")/.."

FIXTURE="tests/fixtures/schema/release-baseline.sql"
[ -f "$FIXTURE" ] || {
    echo "check-schema-baseline: $FIXTURE is missing — run 'just schema-baseline' and commit it" >&2
    exit 1
}

version="$(awk '/^\[workspace\.package\]/{p=1;next}/^\[/{p=0}p&&/^version[[:space:]]*=/{gsub(/[[:space:]"]/,""); sub(/^version=/,""); print; exit}' Cargo.toml)"
[ -n "$version" ] || { echo "check-schema-baseline: could not read the workspace version" >&2; exit 1; }

recorded="$(sed -n '1s/^-- systemprompt-astound release-baseline: \([0-9][0-9.]*\) .*/\1/p' "$FIXTURE")"
[ -n "$recorded" ] || {
    echo "check-schema-baseline: $FIXTURE line 1 is not a release-baseline header" >&2
    exit 1
}

if [ "$recorded" != "$version" ]; then
    echo "check-schema-baseline: fixture records $recorded but the workspace version is $version" >&2
    echo "Run 'just schema-baseline' after the version bump and commit the result." >&2
    exit 1
fi

if grep -q '^\\' "$FIXTURE"; then
    echo "check-schema-baseline: $FIXTURE contains psql meta-commands (lines starting with '\\'); re-record with 'just schema-baseline'" >&2
    exit 1
fi

echo "check-schema-baseline: fixture records $recorded == workspace $version"
