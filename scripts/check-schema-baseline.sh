#!/usr/bin/env bash
# Gate: tests/fixtures/schema/release-baseline-<version>.sql is the upgrade
# ladder — one recorded schema per release from the floor up — and its top
# rung must name the workspace version.
#
# The upgrade test (tests/integration/schema-upgrade) restores every rung
# into an empty database, runs the current installer over it, and diffs the
# result against a fresh install. That only proves anything if the rungs
# really are the released schemas. During a `next` cycle the workspace
# version is still the last release's, so this passes and the test upgrades
# *from* that release; `just core-bump X.Y.Z` bumps the version and turns
# this gate red until `just schema-baseline` records the rung for X.Y.Z —
# which is exactly the moment it has to exist, before `just release`.
#
# Why a ladder and not the previous release alone: the 2026-09-21 self-host
# upgrade failed on core migrations that every N-1 → N test passed, because
# the database being upgraded was older than N-1. Every release tag from the
# floor must have a rung, so the ladder cannot silently go stale.
#
# Why a recorded fixture and not a download or a worktree build: the 0.52
# preview crash-looped on production (2026-09-14) on a declarative schema that
# installed cleanly on every fresh database and every local clone; the only
# database shaped like a deployed install was production. A tracked dump of
# the release is deterministic, offline, and identical in CI and locally.
#
# Why the floor is 0.61.0: it is the first release this repo records a rung
# for. Older installs (the last published release before it is 0.49.0)
# upgrade by first moving to 0.61.0, which is the oldest schema the upgrade
# test can prove.
#
# Header: line 1 is `-- systemprompt-<repo> release-baseline: X.Y.Z (core
# vX.Y.Z)`, as `just schema-baseline` writes it. The retired single fixture
# (release-baseline.sql, no version in its name) is not a rung and is ignored.
set -euo pipefail

cd "$(dirname "$0")/.."

DIR="tests/fixtures/schema"
FLOOR="0.61.0"

version="$(awk '/^\[workspace\.package\]/{p=1;next}/^\[/{p=0}p&&/^version[[:space:]]*=/{gsub(/[[:space:]"]/,""); sub(/^version=/,""); print; exit}' Cargo.toml)"
[ -n "$version" ] || { echo "check-schema-baseline: could not read the workspace version" >&2; exit 1; }

top="$DIR/release-baseline-$version.sql"
[ -f "$top" ] || {
    echo "check-schema-baseline: $top is missing — the workspace is $version; run 'just schema-baseline' and commit it" >&2
    exit 1
}

fail=0
for fixture in "$DIR"/release-baseline-*.sql; do
    named="${fixture##*/release-baseline-}"; named="${named%.sql}"
    recorded="$(sed -n '1s/^-- systemprompt-[a-z-]* release-baseline: \([0-9][0-9.]*\) .*/\1/p' "$fixture")"
    if [ -z "$recorded" ]; then
        echo "check-schema-baseline: $fixture line 1 is not a release-baseline header" >&2; fail=1
    elif [ "$recorded" != "$named" ]; then
        echo "check-schema-baseline: $fixture records $recorded but is named $named" >&2; fail=1
    fi
    if grep -q '^\\' "$fixture"; then
        echo "check-schema-baseline: $fixture contains psql meta-commands (lines starting with '\\'); re-record with 'just schema-baseline $named'" >&2; fail=1
    fi
done

# Why: every release since the floor needs a rung; a tag without one means
# a database left by that release is an upgrade path nothing exercises.
# Tags may be absent from a shallow checkout, in which case only the rungs
# present are checked.
for tag in $(git tag --list 'v[0-9]*' 2>/dev/null | sed 's/^v//' | sort -V); do
    [ "$(printf '%s\n%s\n' "$FLOOR" "$tag" | sort -V | head -1)" = "$FLOOR" ] || continue
    [ "$(printf '%s\n%s\n' "$tag" "$version" | sort -V | head -1)" = "$tag" ] || continue
    [ -f "$DIR/release-baseline-$tag.sql" ] || {
        echo "check-schema-baseline: release v$tag has no rung — run 'just schema-baseline $tag' and commit it" >&2; fail=1
    }
done

[ "$fail" -eq 0 ] || exit 1
echo "check-schema-baseline: top rung $version == workspace; $(ls "$DIR"/release-baseline-*.sql | wc -l | tr -d ' ') rungs from $FLOOR"
