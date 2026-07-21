#!/usr/bin/env bash
# No extension source file (excluding tests) may exceed 300 lines. Shared by
# `just file-size` and the quality.yml file-size CI job.
#
# `//!` module-head lines are excluded from the count, matching the core
# repo's `just file-size`. The ceiling is a cohesion proxy for *code*; a file
# must never have to choose between documenting itself and staying under it.
set -euo pipefail

cd "$(dirname "$0")/.."

violations=$(find extensions -name '*.rs' \
    -not -path '*/target/*' \
    -not -path '*/tests/*' \
    -exec awk '!/^\/\/!/ {n[FILENAME]++} END {for (f in n) if (n[f]>300) print n[f], f}' {} +)
if [ -n "$violations" ]; then
    echo "error: extension source file(s) exceed the 300-line ceiling:"
    echo "$violations"
    echo
    echo "Split the file at a clean seam (extract a sibling module)."
    exit 1
fi
echo "All extension sources within the 300-line ceiling."
