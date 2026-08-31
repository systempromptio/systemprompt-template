#!/usr/bin/env bash
# No extension source file (excluding tests) may exceed 300 lines. Shared by
# `just file-size` and the quality.yml file-size CI job.
#
# `//!` module-head lines are excluded from the count, matching the core
# repo's `just file-size`. The ceiling is a cohesion proxy for *code*; a file
# must never have to choose between documenting itself and staying under it.
#
# Why the self-check below: this guard was a one-line `find | xargs awk` that
# printed offenders and exited 0 regardless, so it passed while 49 files sat
# over the limit. Exiting 1 on a violation fixes that; it does not prove the
# gate still *can* fail, and the next `| awk` that swallows an exit status
# would put it back to silent without looking wrong. So every run first plants
# a deliberately over-limit file in a temp tree and asserts the scan trips on
# it. The gate proves it works before it is allowed to report success.
set -euo pipefail

cd "$(dirname "$0")/.."

# Print "<lines> <path>" for every .rs file under $1 whose non-`//!` line count
# exceeds the ceiling. Silent when there are none.
scan() {
    find "$1" -name '*.rs' \
        -not -path '*/target/*' \
        -not -path '*/tests/*' \
        -exec awk '!/^\/\/!/ {n[FILENAME]++} END {for (f in n) if (n[f]>300) print n[f], f}' {} +
}

self_check() {
    local dir
    dir=$(mktemp -d)
    # shellcheck disable=SC2064
    trap "rm -rf '$dir'" RETURN
    # 301 code lines, plus a `//!` head that must NOT count toward the total.
    {
        echo '//! deliberately oversized fixture for the gate self-check'
        for _ in $(seq 301); do echo 'const _X: u8 = 0;'; done
    } > "$dir/oversized.rs"
    if [ -z "$(scan "$dir")" ]; then
        echo "error: check-file-size self-check failed — the scan did not flag a 301-line file." >&2
        echo "The guard cannot detect violations, so a pass from it means nothing." >&2
        exit 1
    fi
    # And a file at the ceiling must NOT trip, or the gate is off by one and
    # every split would be chasing a limit that is not the documented one.
    rm -f "$dir/oversized.rs"
    {
        echo '//! at the ceiling, must pass'
        for _ in $(seq 300); do echo 'const _X: u8 = 0;'; done
    } > "$dir/at_limit.rs"
    if [ -n "$(scan "$dir")" ]; then
        echo "error: check-file-size self-check failed — a 300-line file was flagged." >&2
        echo "The ceiling is off by one against the documented limit." >&2
        exit 1
    fi
}

self_check

violations=$(scan extensions)
if [ -n "$violations" ]; then
    echo "error: extension source file(s) exceed the 300-line ceiling:"
    echo "$violations"
    echo
    echo "Split the file at a clean seam (extract a sibling module)."
    exit 1
fi
echo "All extension sources within the 300-line ceiling (self-check passed)."
