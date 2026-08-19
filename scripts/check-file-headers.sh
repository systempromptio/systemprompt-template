#!/usr/bin/env bash
# Verify every production source opens with a `//!` module head.
#
# The head is where a module states its purpose and its place in the layering.
# Line 1 must be the `//!` head itself or an inner attribute (`#![...]`) —
# nothing else may precede the head. This matches core's check-headers rigor;
# core's BSL-1.1 licence-line requirement is deliberately not ported (this
# repo is not published under core's licence).
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0
while IFS= read -r file; do
    first=$(head -1 "$file")
    case "$first" in
        '//!'*) headed=1 ;;
        '#!['*)
            headed=0
            while IFS= read -r line; do
                case "$line" in
                    '#!['*|'') continue ;;
                    '//!'*) headed=1; break ;;
                    *) break ;;
                esac
            done < <(head -8 "$file")
            ;;
        *) headed=0 ;;
    esac
    if [ "$headed" -ne 1 ]; then
        echo "MISSING/MISPLACED DOC HEAD: $file"
        fail=1
    fi
done < <(git ls-files -co --exclude-standard 'extensions/**/*.rs' 'src/**/*.rs' 'bridge/src/**/*.rs' \
    | grep -v -e '/tests/' -e '/build\.rs$' -e '^tests/')

if [ "$fail" -ne 0 ]; then
    echo
    echo 'Every production source opens with a `//!` head (inner attributes may'
    echo 'precede it, nothing else). Write a purpose line, not a paraphrase of'
    echo 'the items below it.'
    echo "check-file-headers: FAILED"
    exit 1
fi
echo "check-file-headers: OK"
