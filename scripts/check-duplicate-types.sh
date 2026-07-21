#!/usr/bin/env bash
# Report `pub` type names defined in more than one module.
#
# Rust is happy to let two modules each define a `UsageEvent`. Nothing breaks
# until someone reads the code: `use crate::types::UsageEvent` and
# `use ...::chain::UsageEvent` name different shapes, and a call site that
# imports the wrong one fails with a type error that describes neither. The
# names were the same because both were "an event", not because the values
# were interchangeable.
#
# The rule is that a type name is unique across the extension, so the name is
# enough to know what a value is. When two modules genuinely need the same
# concept, that is one type in one place; when they need different concepts,
# they get different names.
#
# Only `pub`/`pub(crate)` items count — a private helper struct cannot be
# confused at a distance. Generic parameters and re-exports are declarations,
# not definitions, and are ignored.
#
# Scoped per crate. Within a crate the module path is the only thing telling
# two same-named types apart, and `use` puts both in one namespace. Across
# crates the crate name is already part of every path, so `content::SearchQuery`
# and `admin::SearchQuery` do not read as the same thing.
#
# Exemption: annotate a deliberate collision with `// lint-ok: duplicate-type`
# on the line above it, and say why.
set -uo pipefail

SRC_DIRS="${SRC_DIRS:-extensions src}"

# The crate a file belongs to: nearest ancestor directory holding a Cargo.toml.
crate_of() {
    local dir; dir="$(dirname "$1")"
    while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
        [ -f "$dir/Cargo.toml" ] && { printf '%s' "$dir"; return; }
        dir="$(dirname "$dir")"
    done
    printf '.'
}

# "crate<TAB>name" -> space-separated list of "file:line" definition sites.
declare -A SITES

for root in $SRC_DIRS; do
    [ -d "$root" ] || continue
    while IFS= read -r hit; do
        file="${hit%%:*}"
        rest="${hit#*:}"
        line="${rest%%:*}"
        text="${rest#*:}"
        # `pub struct Foo`, `pub(crate) enum Foo`, `pub struct Foo<T>` ...
        [[ "$text" =~ ^[[:space:]]*pub(\([a-z]+([[:space:]]+[a-z:]+)?\))?[[:space:]]+(struct|enum)[[:space:]]+([A-Za-z0-9_]+) ]] || continue
        name="${BASH_REMATCH[4]}"
        # A `// lint-ok: duplicate-type` on the preceding line exempts it.
        if [ "$line" -gt 1 ] && sed -n "$((line - 1))p" "$file" | grep -q 'lint-ok: duplicate-type'; then
            continue
        fi
        SITES["$(crate_of "$file")	$name"]+=" $file:$line"
    done < <(grep -rnE '^[[:space:]]*pub(\([a-z]+([[:space:]]+[a-z:]+)?\))?[[:space:]]+(struct|enum)[[:space:]]+' \
                  "$root" --include='*.rs' 2>/dev/null)
done

violations=0
while IFS= read -r key; do
    name="${key#*	}"
    # shellcheck disable=SC2086
    set -- ${SITES[$key]}
    [ "$#" -gt 1 ] || continue
    if [ "$violations" -eq 0 ]; then
        echo "check-duplicate-types: type name(s) defined in more than one module:" >&2
    fi
    violations=$((violations + 1))
    echo "  $name" >&2
    for site in "$@"; do echo "    $site" >&2; done
done < <(printf '%s\n' "${!SITES[@]}" | sort)

if [ "$violations" -gt 0 ]; then
    echo "" >&2
    echo "Rename each for what it carries, or annotate the deliberate one with" >&2
    echo "'// lint-ok: duplicate-type' and a reason." >&2
    exit 1
fi

echo "check-duplicate-types: OK (every pub type name is defined once)"
exit 0
