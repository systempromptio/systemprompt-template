#!/usr/bin/env bash
# Gate: every page spec carries the four mandatory describe blocks.
#
# A page is not covered because something clicks on it. It is covered when its
# content renders, its actions work, its authorization is stated for every
# principal, and it wears the design language. Those four questions are the
# four blocks, and a spec missing one is a page nobody has actually checked.
#
# The template (playwright/tests/pages/_template.spec.ts) documents the shape.
# Read-only, no server, no node — safe in lint-gates.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/playwright/tests/pages"

[ -d "$DIR" ] || { echo "check-spec-shape: no $DIR yet, nothing to check"; exit 0; }

blocks="renders actions authorization design language"
failed=0
checked=0

for spec in "$DIR"/*.spec.ts; do
    [ -e "$spec" ] || continue
    checked=$((checked + 1))
    name="$(basename "$spec")"
    for block in renders actions authorization; do
        grep -qE "test\.describe\(['\"]$block['\"]" "$spec" || {
            echo "check-spec-shape: $name is missing the '$block' describe block"
            failed=1
        }
    done
    grep -qE "test\.describe\(['\"]design language['\"]" "$spec" || {
        echo "check-spec-shape: $name is missing the 'design language' describe block"
        failed=1
    }
    # The density bar is measured, not eyeballed: a page spec that never calls
    # expectDensity leaves the one claim this console is built on unchecked.
    grep -q 'expectDensity' "$spec" || {
        echo "check-spec-shape: $name never calls expectDensity — the density bar is unmeasured"
        failed=1
    }
done

if [ "$failed" -ne 0 ]; then
    echo
    echo "Every playwright/tests/pages/*.spec.ts needs all four blocks ($blocks)"
    echo "and one expectDensity call in the design-language block:"
    echo
    echo "    import { expectAccessible, expectDensity } from '../support/a11y';"
    echo "    await expectDensity(adminPage, 'list');   // or 'detail'"
    echo
    echo "Copy playwright/tests/pages/_template.spec.ts for the shape."
    exit 1
fi

echo "check-spec-shape: OK ($checked page specs, all four blocks present)"
