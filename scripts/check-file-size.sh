#!/usr/bin/env bash
# No extension source file (excluding tests) may exceed 300 lines. Shared by
# `just file-size` and the quality.yml file-size CI job.
set -euo pipefail

cd "$(dirname "$0")/.."

violations=$(find extensions -name '*.rs' \
    -not -path '*/target/*' \
    -not -path '*/tests/*' \
    -exec wc -l {} + | awk '$1>300 && $2!="total"')
if [ -n "$violations" ]; then
    echo "error: extension source file(s) exceed the 300-line ceiling:"
    echo "$violations"
    echo
    echo "Split the file at a clean seam (extract a sibling module)."
    exit 1
fi
echo "All extension sources within the 300-line ceiling."
