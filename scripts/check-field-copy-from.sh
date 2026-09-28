#!/usr/bin/env bash
# Gate wrapper: fail on a `From` impl that only copies fields across.
#
# The Python does the work; wrapped in shell because lint-gates runs
# `bash scripts/<name>`, and because the failure is worth a sentence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! python3 "$ROOT/scripts/check-field-copy-from.py"; then
    echo
    echo "A From impl that restates the same fields is two types where there should"
    echo "be one, and nothing tells you when they drift. core's config::RateLimitConfig"
    echo "was this against profile::RateLimitsConfig: 15 identical fields plus a second"
    echo "set of defaults, and the two validators over them had already diverged — one"
    echo "checked the gateway budget, the other checked the registries, neither checked"
    echo "both. check-duplicate-types.sh cannot see this: the names differ."
    exit 1
fi
