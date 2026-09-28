#!/usr/bin/env bash
# Gate: a Dockerfile may not name a binary or a build-context path that the
# repository no longer contains.
#
# Nothing in gates.yml builds the image, so a `COPY` of a deleted crate's binary
# is invisible to every tier that runs on a push. Removing the knowledge-bank
# and requirements-workflow extensions left both Dockerfiles still building and
# copying them: all four CI jobs passed, `main` fast-forwarded, and release.yml
# was the first thing to notice. This check is the cheap part of that build.
#
# Two rules:
#   1. every `systemprompt-mcp-<name>` a Dockerfile OR a workflow names must be
#      declared as `binary:` by some extensions/mcp/*/manifest.yaml. Workflows
#      count because release.yml's smoke step asserts the binaries are in the
#      image (`docker run --entrypoint ls ... /app/bin/<binary>`), so a deleted
#      one fails the release there as surely as in the build.
#   2. in the ROOT Dockerfile only, every COPY source must be TRACKED. Its
#      build context is the repository root (`COPY . /src`), so a path there is
#      checkable — but it must be checked against git, not the disk: release.yml
#      builds from a fresh checkout, so an untracked leftover (a __pycache__
#      surviving a `git rm`) would make a broken COPY look fine locally and fail
#      only in the release. Every other image is built with its own directory as
#      the context, which a Dockerfile does not state, so guessing would only
#      produce false positives — they get rule 1 alone.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

declared=$(grep -h '^[[:space:]]*binary:' extensions/mcp/*/manifest.yaml 2>/dev/null |
    sed 's/.*binary:[[:space:]]*//' | tr -d '"' | sort -u)

while IFS= read -r file; do
    [ -f "$file" ] || continue

    for binary in $(grep -oE 'systemprompt-mcp-[a-z0-9-]+' "$file" | sort -u); do
        if ! printf '%s\n' "$declared" | grep -qx "$binary"; then
            echo "$file: names '$binary', which no extensions/mcp/*/manifest.yaml declares"
            fail=1
        fi
    done

    # Rule 2 is root-only: see the header.
    [ "$file" = "Dockerfile" ] || continue

    # COPY [--flags] <src>... <dest>: every src but the final field, skipping
    # any line that takes its source from a previous build stage.
    while IFS= read -r line; do
        case "$line" in *--from=*) continue ;; esac
        # shellcheck disable=SC2086
        set -- $(printf '%s' "$line" | sed 's/^[[:space:]]*COPY[[:space:]]*//; s/--[a-z-]*=[^[:space:]]*//g')
        [ "$#" -ge 2 ] || continue
        count=$(($# - 1))
        for src in "$@"; do
            [ "$count" -gt 0 ] || break
            count=$((count - 1))
            case "$src" in /*|\$*|"") continue ;; esac
            # Tracked, not merely present: a glob that matches no tracked file
            # is as broken as a missing literal.
            if [ -z "$(git ls-files -- "$src" | head -1)" ]; then
                echo "$file: COPY source '$src' is not tracked, so it will not exist in the release build context"
                fail=1
            fi
        done
    done < <(grep -E '^[[:space:]]*COPY[[:space:]]' "$file")
done < <(git ls-files '*Dockerfile' '*Dockerfile.*' '.github/workflows/*.yml' | grep -v '\.bak')

if [ "$fail" -ne 0 ]; then
    echo ""
    echo "A Dockerfile or workflow still names something the repository does not have."
    echo "gates.yml never builds the image, so this only surfaces at release."
    exit 1
fi
echo "check-dockerfile-paths: OK (every Dockerfile and workflow names only what exists)"
