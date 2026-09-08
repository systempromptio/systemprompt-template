#!/usr/bin/env bash
# Gate wrapper: fail on a Handlebars reference that resolves to nothing.
#
# The Python does the work and reports by default; --strict is what makes it a
# gate. Wrapped in shell because lint-gates runs `bash scripts/<name>` and
# because the failure deserves a sentence of its own about why it matters.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! python3 "$ROOT/scripts/check-template-fields.py" --strict; then
    echo
    echo "A template reads a field nothing defines. Handlebars strict mode makes a"
    echo "missing TOP-LEVEL field a 500, but a bad path inside an {{#each}} renders"
    echo "as empty string — so the page looks right with one value silently absent,"
    echo "and a spec asserting the page renders passes."
    exit 1
fi
