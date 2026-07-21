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

# The same defect, one level up: a handler that returns a bare `Response` has
# no error channel at all, so it must build every failure by hand — which is
# how status codes and bodies drifted apart in the first place. Handlers return
# `AdminResult<Response>` (JSON) or `AdminHtmlResult<Response>` (pages); axum's
# `impl IntoResponse for Result<T, E>` means the route table does not care.
HANDLER_DIR="extensions/web/admin/src/handlers"

# An exemption counts if `lint-ok: http-error` sits on the offending line, in
# the three lines above it, or on the line below. It has to accept all three
# because rustfmt relocates a long trailing comment — off the signature and
# onto the first line of the body. A same-line-only rule would pass review and
# then fail the next time anyone ran `cargo fmt`.
scan() {
    local pattern="$1"
    local f
    while IFS= read -r f; do
        awk -v pat="$pattern" -v fname="$f" '
            { line[FNR] = $0; n = FNR }
            END {
                for (i = 1; i <= n; i++) {
                    if (line[i] !~ pat) continue
                    lo = i - 3; if (lo < 1) lo = 1
                    ok = 0
                    for (j = lo; j <= i + 1; j++) {
                        if (line[j] ~ /lint-ok: http-error/) ok = 1
                    }
                    if (!ok) printf "%s:%d:%s\n", fname, i, line[i]
                }
            }' "$f"
    done < <(find "$HANDLER_DIR" -name '*.rs')
}

UNTYPED=$(scan '->[[:space:]]*(impl IntoResponse|Response)[[:space:]]*[{]')

if [ -n "$UNTYPED" ]; then
    echo "check-http-errors: handler returns an untyped response."
    echo "Return AdminResult<Response>, or AdminHtmlResult<Response> for a page, so the"
    echo "error variant picks the status. A helper that genuinely renders a response"
    echo "rather than an error may annotate it with '// lint-ok: http-error':"
    echo "$UNTYPED"
    exit 1
fi

# Statuses come from an `AdminError` variant, never a literal. These two are
# the ones that were being hand-written most often, and the ones where getting
# it wrong is least visible: a 500 that should have been a 404, or a 403 built
# beside the check instead of by it.
LITERALS=$(scan 'StatusCode::(INTERNAL_SERVER_ERROR|FORBIDDEN)')

if [ -n "$LITERALS" ]; then
    echo "check-http-errors: raw status literal in a handler."
    echo "Use AdminError::Forbidden / AdminError::internal so the classification and the"
    echo "logging happen in one place, or annotate a deliberate exception with"
    echo "'// lint-ok: http-error':"
    echo "$LITERALS"
    exit 1
fi
