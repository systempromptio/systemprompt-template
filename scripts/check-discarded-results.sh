#!/usr/bin/env bash
# Gate: core's rust-contracts scanner in `discarded` mode over this repo's
# production roots. The rules and the tool live in systemprompt-core
# (scripts/check-discarded-results.sh), so this wraps the sibling checkout rather than
# carrying a copy that would drift from it.
#
# Locally the sibling is optional: without ../systemprompt-core the gate says
# so and passes (`just core-checkout` clones it). In CI it is mandatory —
# gates.yml checks core out at bridge/CORE_REF — so an absent sibling there is
# a failure, never a quiet skip.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
script="$root/../systemprompt-core/scripts/check-discarded-results.sh"
if [ ! -f "$script" ]; then
    if [ -n "${CI:-}" ]; then
        echo "check-discarded-results: no sibling core at $root/../systemprompt-core (CI must run .github/actions/core-checkout first)" >&2
        exit 1
    fi
    echo "check-discarded-results: SKIPPED — no sibling core checkout at ../systemprompt-core (run: just core-checkout)"
    exit 0
fi
exec bash "$script" "$root/extensions"
