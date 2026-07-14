#!/usr/bin/env bash
# Reject `let _ = <expr>.unwrap()/.expect()` in the test workspace.
#
# Discarding the result of a fallible call runs the code for its panic-on-error
# side effect but asserts nothing about what it produced — a test that can only
# fail by panicking, never by a wrong value. Bind the result and assert on it,
# or drop the `let _ =` and let the expression stand.
#
# Exemption: a line that genuinely exercises a side effect with nothing to
# assert (a coverage driver, a warm-up call) may annotate it with
# `// lint-ok: no-assert` followed by a reason.
set -uo pipefail

SEARCH_DIR="tests"

if [ ! -d "$SEARCH_DIR" ]; then
    echo "check-test-value: no $SEARCH_DIR workspace yet - nothing to check"
    exit 0
fi

# Rule 1: `let _ = <expr>.unwrap()/.expect()` — panic-only, asserts nothing.
PATTERN_PANIC='^\s*let _ = .*\.(unwrap|expect)\('

# Rule 2: `let _ = <expr>.len()/.count();` — discards a query/collection size
# for no observable effect. A DB-backed `.len()` discard runs the query but
# asserts nothing about the rows; on a scoped query, bind it and assert
# seeded-row membership, otherwise a per-row property. This class is invisible
# to rule 1 (no `.unwrap()`), so it is caught separately.
PATTERN_SIZE='^\s*let _ = [A-Za-z_][A-Za-z0-9_.]*\.(len|count)\(\)\s*;'

if ! command -v rg >/dev/null 2>&1; then
    echo "check-test-value: ripgrep (rg) is required" >&2
    exit 2
fi

# Whole-file exclusions: subprocess coverage drivers that legitimately invoke
# the CLI for its side effect and assert elsewhere (or not at all, by design).
filter_hits() {
    grep -v 'lint-ok: no-assert' \
        | grep -vE '/(subprocess_smoke|subprocess_with_db|subprocess_full)\.rs:' \
        | grep -v '^[[:space:]]*$' || true
}

STATUS=0

RAW_PANIC=$(rg -n --no-heading --color=never -g '*.rs' "$PATTERN_PANIC" "$SEARCH_DIR" 2>/dev/null || true)
HITS_PANIC=$(printf '%s\n' "$RAW_PANIC" | filter_hits)
if [ -n "$HITS_PANIC" ]; then
    echo "check-test-value: \`let _ = ….unwrap()/.expect()\` in a test — panic-only, asserts nothing."
    echo "Bind the result and assert on it, or annotate a deliberate side-effect call"
    echo "with '// lint-ok: no-assert <reason>':"
    echo "$HITS_PANIC"
    STATUS=1
fi

RAW_SIZE=$(rg -n --no-heading --color=never -g '*.rs' "$PATTERN_SIZE" "$SEARCH_DIR" 2>/dev/null || true)
HITS_SIZE=$(printf '%s\n' "$RAW_SIZE" | filter_hits)
if [ -n "$HITS_SIZE" ]; then
    echo "check-test-value: \`let _ = ….len()/.count();\` in a test — discards a size, asserts nothing."
    echo "Bind the value and assert a real contract (seeded-row membership on a scoped"
    echo "query, or a per-row property), or annotate with '// lint-ok: no-assert <reason>':"
    echo "$HITS_SIZE"
    STATUS=1
fi

if [ "$STATUS" -ne 0 ]; then
    exit 1
fi

echo "check-test-value: OK — no unasserted discarded results in $SEARCH_DIR"
