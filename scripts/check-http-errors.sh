#!/usr/bin/env bash
# Reject inline `map_err(|e| ApiError::ctor(...))` at HTTP call sites.
#
# HTTP status mapping belongs in an entry-local error type's `From` impls
# (see each crate error.rs / error/ module);
# call sites propagate with bare `?` so the variant decides the status. An inline
# closure flattens distinct failure modes into one status (a dropped DB
# connection becomes a 404) and discards the typed cause.
#
# Exemption: a deliberate variant re-classification, or adapting a foreign
# variant-less error, may annotate the line with `// lint-ok: http-error`.
set -uo pipefail

SEARCH_DIR="extensions"
PATTERN='map_err\(\s*\|[^|]*\|\s*ApiError::'

if ! command -v rg >/dev/null 2>&1; then
    echo "check-http-errors: ripgrep (rg) is required" >&2
    exit 2
fi

RAW=$(rg -Un --multiline-dotall --no-heading --color=never \
    -g '*.rs' \
    "$PATTERN" "$SEARCH_DIR" 2>/dev/null || true)

HITS=$(printf '%s\n' "$RAW" | grep -v 'lint-ok: http-error' | grep -v '^[[:space:]]*$' || true)

if [ -n "$HITS" ]; then
    echo "check-http-errors: inline map_err into ApiError at an HTTP call site."
    echo "Map domain -> HTTP via an entry-local error type's From impls and propagate"
    echo "with bare ?, or annotate a deliberate re-classification with '// lint-ok: http-error':"
    echo "$HITS"
    exit 1
fi
