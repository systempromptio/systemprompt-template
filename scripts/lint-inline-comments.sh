#!/usr/bin/env bash
set -uo pipefail

# Machete rule: inline `//` comments are banned in production crates.
#
# The only permitted full-line inline comments are the two whitelisted
# justification prefixes mandated by the rust-coding-standards skill:
#
#   // Why:    — a non-obvious invariant, hidden constraint, or exemption
#                justification (e.g. a permitted `let _ =`)
#   // JSON:   — a sanctioned `serde_json::Value` protocol-boundary usage
#   // SAFETY: — the discharge of an `unsafe` block's obligations. Not a
#                discretionary comment: clippy's `undocumented_unsafe_blocks`
#                is warn-level and the workspace builds with `-D warnings`, so
#                every `unsafe` block must carry one and it must start with
#                `SAFETY:` for clippy to recognise it. Banning the prefix here
#                would leave the two gates mutually unsatisfiable.
#
# Continuation lines of a whitelisted comment block are allowed. `//!` module
# heads are governed separately (rustdoc placement rules), as are `///` docs
# on public API items. `tests/**` and `build.rs` files are out of scope.
#
# A second check enforces rustdoc placement: `///` is reserved for public
# traits, top-level types (struct/enum/union/type), and `mod` declarations.
# Rustdoc on anything else — functions, methods, consts, statics, fields,
# enum variants, macros, or any `pub(crate)`/`pub(super)`/private item — is
# banned. A genuine invariant on such an item belongs in a `// Why:` comment;
# anything else is deleted.
#
# Exemption: a path (one per line, `#` comments) in
# scripts/rustdoc-exemptions.txt. Reserve it for files where `///` is
# load-bearing rather than decorative — clap-derive CLIs, where the rustdoc
# *is* the generated `--help` text and deleting it silently guts the CLI.
#
# Scope: production sources in `extensions/**`, `src/**` and `bridge/src/**`, tracked or
# not (`git ls-files -co`) — an untracked new file must not pass vacuously.

EXEMPT_FILE="scripts/rustdoc-exemptions.txt"
is_exempt() {
    [ -f "$EXEMPT_FILE" ] || return 1
    grep -v '^[[:space:]]*#' "$EXEMPT_FILE" | grep -qxF "$1"
}

MATCHES=""
while IFS= read -r file; do
    case "$file" in
        tests/*|*/tests/*) continue ;;
        */build.rs) continue ;;
    esac
    is_exempt "$file" && continue
    FOUND=$(awk '
        /^[[:space:]]*\/\/\// { prev_allowed = 0; if (!in_doc) doc_line = FNR; in_doc = 1; next }
        /^[[:space:]]*\/\/!/ { prev_allowed = 0; next }
        /^[[:space:]]*\/\// {
            in_doc = 0
            if ($0 ~ /^[[:space:]]*\/\/ (Why|JSON|SAFETY):/) { prev_allowed = 1; next }
            if (prev_allowed) { next }
            print FILENAME ":" FNR ":" $0
            next
        }
        /^[[:space:]]*#!?\[/ {
            open = gsub(/\[/, "["); close_n = gsub(/\]/, "]")
            if (open > close_n) { in_attr = 1; attr_depth = open - close_n }
            next
        }
        {
            if (in_attr) {
                attr_depth += gsub(/\[/, "[") - gsub(/\]/, "]")
                if (attr_depth <= 0) { in_attr = 0 }
                next
            }
            if ($0 ~ /^[[:space:]]*$/) { next }
            if (in_doc) {
                stripped = $0
                sub(/^[[:space:]]+/, "", stripped)
                if (stripped !~ /^(pub[[:space:]]+)?(unsafe[[:space:]]+)?(trait|struct|enum|union|type|mod)[[:space:]<]/) {
                    print FILENAME ":" doc_line ": rustdoc on non-type item (" stripped ") — /// is reserved for pub traits, top-level types, and modules; use // Why: or delete"
                } else if (stripped !~ /^pub[[:space:]]/) {
                    print FILENAME ":" doc_line ": rustdoc on non-public item (" stripped ") — use // Why: or delete"
                }
            }
            in_doc = 0
            prev_allowed = 0
        }
    ' "$file")
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard 'extensions/*.rs' 'extensions/**/*.rs' 'src/*.rs' 'src/**/*.rs' 'bridge/src/*.rs' 'bridge/src/**/*.rs' | sort -u)

# `///` rustdoc is banned in test code (core rule, previously only logged by
# the observational audit): rustdoc is never rendered for test crates, so a
# doc there is a paraphrase by definition. Scaffolding `//` comments stay
# legal; `//!` heads are optional and legal.
while IFS= read -r file; do
    FOUND=$(grep -n '^[[:space:]]*///' "$file" \
        | sed "s|^|${file}:|;s|\$| — /// banned in test code, use //|" || true)
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard 'tests/**/*.rs' ':(glob)**/tests/**/*.rs' | sort -u)

if [ -z "$MATCHES" ]; then
    echo "lint-inline-comments: OK (no unlisted inline comments)"
    exit 0
fi

echo "lint-inline-comments: inline // comments are banned in production crates,"
echo "and /// rustdoc is reserved for pub traits, top-level types, and modules."
echo "Delete the comment, or justify it with a '// Why:', '// JSON:' or '// SAFETY:' prefix:"
echo ""
printf '%s' "$MATCHES"
exit 1
