#!/usr/bin/env bash
# Gate: a migration file may never reuse a number an established database has
# already applied.
#
# The web migration chain was retired in ab902af2 (files 002-027 deleted,
# declarative schema + seeds became the bootstrap), but every database that ran
# those migrations still carries a ledger row for each number. Refilling a
# retired slot makes the recorded checksum disagree with the file, and the
# migrator refuses to boot — the whole deployment crash-loops on a mismatch that
# names a migration nobody recognises.
#
# Each extension records its high-water mark in schema/migrations/.retired-through.
# New migrations take the next number above it.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
while IFS= read -r marker; do
    dir=$(dirname "$marker")
    ext=$(printf '%s' "$dir" | sed -E 's|^extensions/([^/]+)/.*|\1|')
    retired=$(tr -cd '0-9' < "$marker")
    [ -n "$retired" ] || { echo "$marker: no number recorded"; fail=1; continue; }
    for f in "$dir"/[0-9]*.sql; do
        [ -e "$f" ] || continue
        n=$(basename "$f" | sed -E 's/^0*([0-9]+)_.*/\1/')
        if [ "$n" -le "$retired" ]; then
            echo "$ext: $(basename "$f") reuses migration number $n, retired through $retired"
            echo "  Renumber it above $retired — established databases have already spent that slot."
            fail=1
        fi
    done
done < <(find extensions -path '*/schema/migrations/.retired-through' | sort)

if [ "$fail" -eq 0 ]; then
    echo "check-migration-numbers: ok"
fi
exit "$fail"
