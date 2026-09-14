#!/usr/bin/env bash
# Gate: a migration file may never reuse a number an established database has
# already applied.
#
# A slot is spent whether or not its file is still in the tree. Refilling one
# leaves the new SQL unexecuted -- the runner sees the number as applied and
# skips it -- while the recorded checksum belongs to a migration nobody can
# find, so the boot refuses on a mismatch that names something unrecognisable.
# The new pages simply never work, and nothing says why.
#
# Spent slots are recorded as `NNN_name.tombstone` (or `NNN-MMM_name.tombstone`
# for a retired span) beside the migrations themselves, so that `ls` on this
# directory tells the truth about which numbers are free. Deleting a migration
# is only half the job: a6a57f3f deleted 034_knowledge_bank.sql and recorded the
# number nowhere, which left 034 looking free while every database from v0.42.0
# on had it applied.
#
# The number is taken from the FILENAME. A tombstone's body is documentation for
# humans and is never load-bearing.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
while IFS= read -r dir; do
    ext=$(printf '%s' "$dir" | sed -E 's|^extensions/([^/]+)/.*|\1|')

    burned=" "
    spent=0
    for t in "$dir"/*.tombstone; do
        [ -e "$t" ] || continue
        stem=$(basename "$t" .tombstone)
        range=${stem%%_*}
        case "$range" in
            [0-9]*-[0-9]*)
                lo=$((10#${range%%-*}))
                hi=$((10#${range##*-}))
                if [ "$lo" -gt "$hi" ]; then
                    echo "$ext: $(basename "$t") has a reversed span ($lo-$hi)"
                    echo "  Write it low-to-high, or the slots it claims are silently none."
                    fail=1
                    continue
                fi
                n=$lo
                while [ "$n" -le "$hi" ]; do
                    burned="$burned$n "
                    n=$((n + 1))
                done
                spent=$((spent + hi - lo + 1))
                ;;
            *[!0-9]*|"")
                echo "$ext: $(basename "$t") carries no migration number in its filename"
                echo "  Name it NNN_name.tombstone or NNN-MMM_name.tombstone; the body is not read."
                fail=1
                ;;
            *)
                burned="$burned$((10#$range)) "
                spent=$((spent + 1))
                ;;
        esac
    done

    for f in "$dir"/[0-9]*.sql; do
        [ -e "$f" ] || continue
        n=$(basename "$f" | sed -E 's/^0*([0-9]+)_.*/\1/')
        n=$((10#$n))
        if [ "${burned#* $n }" != "$burned" ]; then
            echo "$ext: $(basename "$f") reuses spent migration number $n"
            echo "  A tombstone records that slot as shipped; established databases have it applied."
            echo "  Renumber above the highest number in this directory."
            fail=1
        fi
    done

    # Why: printed so an empty set is visible rather than silent. A gate that
    # enforces nothing looks identical to a gate that passes.
    echo "check-migration-numbers: $ext — $spent spent slot(s) enforced"
done < <(find extensions -type d -path '*/schema/migrations' | sort)

if [ "$fail" -eq 0 ]; then
    echo "check-migration-numbers: ok"
fi
exit "$fail"
