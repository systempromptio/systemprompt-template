#!/usr/bin/env bash
# Reject archaeology, caller-reference, and paraphrase comments.
# Exemption: annotate the line above with `// doc-ok: <reason>`.
set -uo pipefail

SEARCH_DIRS=(extensions src)

if ! command -v rg >/dev/null 2>&1; then
    echo "check-comments: ripgrep (rg) is required" >&2
    exit 2
fi

fail=0

# Lines under a `# Errors` heading are required by clippy::missing_errors_doc.
strip_errors_sections() {
    awk -F: '
        {
            file = $1; line = $2
            cmd = "sed -n " (line - 1) "p " file
            cmd | getline prev
            close(cmd)
            if (prev ~ /^[[:space:]]*\/\/\/[[:space:]]*#[[:space:]]*Errors/) next
            if (prev ~ /doc-ok:/) next
            print
        }
    '
}

report() {
    local label="$1" remedy="$2" hits="$3"
    [ -z "$hits" ] && return
    echo "check-comments: $label"
    echo "  $remedy"
    echo "$hits"
    echo
    fail=1
}

ARCHAEOLOGY=$(rg -n --no-heading --color=never -g '*.rs' -g '!**/tests/**' \
    '^\s*(///|//)[^!].*(\(was |historical|previously|no longer|used to be|formerly|renamed)' \
    "${SEARCH_DIRS[@]}" 2>/dev/null | grep -v 'doc-ok:' || true)
report "comment narrates how the code used to look." \
    "State current behaviour only — history lives in git." \
    "$ARCHAEOLOGY"

CALLER_REF=$(rg -n --no-heading --color=never -g '*.rs' -g '!**/tests/**' \
    '^\s*///.*(Used by |Consumed by |Called by |Read by )' \
    "${SEARCH_DIRS[@]}" 2>/dev/null | grep -v 'doc-ok:' || true)
report "doc names a downstream caller." \
    "Callers move; describe what this item guarantees instead." \
    "$CALLER_REF"

PARAPHRASE=$(rg -n --no-heading --color=never -g '*.rs' -g '!**/tests/**' \
    '^\s*///\s*(Returns|Fetches|Inserts|Deletes|Updates|Creates|Builds|Gets|Constructs|Computes?|Wraps) ' \
    "${SEARCH_DIRS[@]}" 2>/dev/null | strip_errors_sections || true)
report "doc restates the signature." \
    "Delete it, or rewrite to carry the constraint the signature cannot encode." \
    "$PARAPHRASE"

if [ "$fail" -ne 0 ]; then
    exit 1
fi

echo "check-comments: OK — no archaeology, caller-reference, or paraphrase comments."
