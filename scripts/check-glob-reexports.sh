#!/usr/bin/env bash
# Reject `pub use <module>::*;` in extension sources.
#
# A glob re-export makes a module's public surface unknowable without opening
# every child, and it silently merges namespaces: two domains that each export
# `queries` or `list_agents` collide at the parent, and the fix people reach for
# is an alias (`create_route as create_gateway_route`) that hides which module a
# symbol actually came from. Name the items, or let callers path-qualify.
#
# Exemption: annotate a deliberate prelude-style re-export with
# `// lint-ok: glob-reexport` and a reason on the preceding line.
set -uo pipefail

SEARCH_DIRS=(extensions src)
PATTERN='^\s*pub use [A-Za-z0-9_:]+::\*;'

RAW=""
for d in "${SEARCH_DIRS[@]}"; do
    [ -d "$d" ] || continue
    if command -v rg >/dev/null 2>&1; then
        RAW+=$(rg -n --no-heading --color=never -g '*.rs' \
            -g '!**/target/**' -g '!**/.sqlx/**' \
            -e "$PATTERN" "$d" 2>/dev/null || true)$'\n'
    else
        RAW+=$(grep -rnE --include='*.rs' "$PATTERN" "$d" 2>/dev/null \
            | grep -v '/target/' || true)$'\n'
    fi
done

HITS=$(printf '%s\n' "$RAW" | grep -v 'lint-ok: glob-reexport' | grep -v '^[[:space:]]*$' || true)

if [ -n "$HITS" ]; then
    echo "check-glob-reexports: glob re-export found."
    echo "List the items explicitly, or drop the re-export and let callers"
    echo "path-qualify. Annotate a deliberate prelude with '// lint-ok: glob-reexport':"
    echo "$HITS"
    exit 1
fi

echo "check-glob-reexports: OK (no glob re-exports)"
