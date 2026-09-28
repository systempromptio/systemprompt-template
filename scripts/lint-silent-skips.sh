#!/usr/bin/env bash
# Fails when a test function returns early on a missing prerequisite without
# saying so.
#
# A test that returns instead of running reports the same green as a test that
# ran, so a whole tier can pass with the database down. The sanctioned way to
# give up is a helper that panics under CI -- `TempDb::create`, `db_or_skip!`,
# `fixture_database_url` -- which is why a site naming one is allowed. Anything
# else needs `// skip-ok: <reason>` on the line, which makes the decision
# visible in review rather than invisible in a green run.
#
# Portable to macOS and Linux: POSIX awk only, no grep -P, no head -n -1.
set -euo pipefail

cd "$(dirname "$0")/.."

roots=("$@")
if [ ${#roots[@]} -eq 0 ]; then
    if [ -d crates/tests ]; then roots=(crates/tests); else roots=(tests); fi
fi

for root in "${roots[@]}"; do
    if [ ! -d "$root" ]; then
        echo "lint-silent-skips: no such test root: $root" >&2
        exit 1
    fi
done

findings=$(find "${roots[@]}" -name '*.rs' -type f -not -path '*/target/*' -print0 \
    | xargs -0 awk '
FNR == 1 { in_test = 0; pending = 0; holding = 0; n = 0 }

# A test body runs from the attribute-marked `fn` at column 0 to the closing
# brace at column 0. Anything above it is a helper, and a helper returning
# None is how a gateway reports a missing prerequisite.
/^#\[(tokio::)?test\]/ || /^#\[test\(/ { pending = 1 }
/^(pub )?(async )?fn / { if (pending) { in_test = 1 }; pending = 0 }
/^}/ { in_test = 0 }

{
    # Why: the binding that names the gate can sit several lines above its
    # `else {` when the call is wrapped, so the check reads the statement, not
    # the line.
    prev = hist[n % 8]
    n++
    hist[n % 8] = $0
    if (!in_test) { holding = 0; next }

    silent = 0
    lineno = FNR
    if ($0 ~ /else[ \t]*\{[ \t]*return[ \t]*;?[ \t]*\}/) { silent = 1 }
    if ($0 ~ /else[ \t]*\{[ \t]*$/) { holding = FNR; next }
    if (holding == FNR - 1 && $0 ~ /^[ \t]*return[ \t]*;[ \t]*$/) { silent = 1; lineno = holding }
    holding = 0
    if (!silent) { next }

    stmt = ""
    for (k = 0; k < 8; k++) { stmt = stmt " " hist[(n + k) % 8] }
    if (stmt ~ /skip-ok:/) { next }
    if (prev ~ /skip-ok:/) { next }
    if (stmt ~ /_or_skip/) { next }
    if (stmt ~ /fixture_database_url/ || stmt ~ /fixture_db_pool/) { next }
    if (stmt ~ /db_pool_or_skip/ || stmt ~ /TempDb::create/ || stmt ~ /db_or_skip/) { next }
    printf "%s:%d:%s\n", FILENAME, lineno, hist[n % 8]
}
') || true

if [ -n "$findings" ]; then
    printf '%s\n' "$findings"
    count=$(printf '%s\n' "$findings" | awk 'END { print NR }')
    cat >&2 <<MSG

lint-silent-skips: $count test(s) return early without declaring why.

A test that skips silently is indistinguishable from a test that passed. Take
one of these instead:
  - acquire the prerequisite through a gate that panics under CI
    (\`TempDb::create\`, \`db_or_skip!()\`, \`fixture_database_url\`);
  - annotate the line \`// skip-ok: <reason>\` when the prerequisite genuinely
    cannot exist on a developer machine.
MSG
    exit 1
fi

echo "lint-silent-skips: no undeclared silent skips"
