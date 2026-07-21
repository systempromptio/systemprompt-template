#!/usr/bin/env bash
# Verify every production extension source opens with a `//!` module head.
#
# The head is where a module states its purpose and its place in the layering.
# Unlike systemprompt-core, the template carries no per-file license reference
# (the repo is MIT and licenses at the root), so only the doc head is asserted.
#
# Inner attributes (`#![allow(...)]`) may precede the head, so the check scans
# the first few lines rather than only line 1.
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0
while IFS= read -r file; do
    if ! head -6 "$file" | grep -q '^//!'; then
        echo "MISSING DOC HEAD: $file"
        fail=1
    fi
done < <(find extensions src -name '*.rs' \
    -not -path '*/target/*' \
    -not -path '*/tests/*' \
    ! -name build.rs 2>/dev/null)

if [ "$fail" -ne 0 ]; then
    echo
    echo 'Every production source needs a `//!` head stating what the module is'
    echo 'for. Write a purpose line, not a paraphrase of the items below it.'
    echo "check-file-headers: FAILED"
    exit 1
fi
echo "check-file-headers: OK"
