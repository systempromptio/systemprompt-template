#!/usr/bin/env bash
# Pre-merge gate keeping SQL out of extension.rs files.
#
# Schema DDL and migration SQL live in <crate>/schema/**.sql and are embedded
# via include_str!() (schemas) or generated from filenames by the build script
# (migrations). An extension.rs must never carry SQL as a Rust string literal,
# and must never hand-build Migration values — migrations are discovered from
# schema/migrations/NNN_<name>.sql and surfaced by extension_migrations!().
#
# A file counts as a violation if it contains either:
#   - a raw string literal (r"..." / r#"..."#) — the form inline SQL takes; or
#   - a Migration::new / Migration::with_down / Migration::new_no_transaction
#     call — migrations must come from extension_migrations!().

set -euo pipefail

ROOT="${1:-extensions}"

if ! command -v grep >/dev/null 2>&1; then
    echo "lint-extensions: grep not found" >&2
    exit 2
fi

# shellcheck disable=SC2207
files=($(find "$ROOT" -type f -name 'extension.rs' \
    -not -path '*/target/*' \
    | sort))

if [ ${#files[@]} -eq 0 ]; then
    echo "lint-extensions: no extension.rs files under $ROOT"
    exit 0
fi

# A raw-string literal opener: a lowercase `r`, optional `#`s, then `"`, where
# the `r` is not part of a preceding identifier (which would make it the tail
# of a word followed by an ordinary string).
raw_string='(^|[^A-Za-z0-9_])r#*"'
migration_ctor='Migration::(new|with_down|new_no_transaction)\b'
violations=0

for f in "${files[@]}"; do
    file_violations=0

    if matches=$(grep -nE "$raw_string" "$f" || true); [ -n "$matches" ]; then
        while IFS= read -r line; do
            echo "$f:$line: raw string literal — SQL belongs in <crate>/schema/**.sql"
            file_violations=$((file_violations + 1))
        done <<< "$matches"
    fi

    if matches=$(grep -nE "$migration_ctor" "$f" || true); [ -n "$matches" ]; then
        while IFS= read -r line; do
            echo "$f:$line: hand-built Migration — declare migrations as \
schema/migrations/NNN_<name>.sql and return extension_migrations!()"
            file_violations=$((file_violations + 1))
        done <<< "$matches"
    fi

    if [ $file_violations -gt 0 ]; then
        violations=$((violations + 1))
    fi
done

if [ $violations -gt 0 ]; then
    echo "" >&2
    echo "lint-extensions: $violations file(s) carry SQL or hand-built migrations." >&2
    echo "Move schema DDL to <crate>/schema/<table>.sql (include_str!)." >&2
    echo "Move migration SQL to <crate>/schema/migrations/NNN_<name>.sql; the" >&2
    echo "build script discovers it and extension_migrations!() returns it." >&2
    exit 1
fi

echo "lint-extensions: ${#files[@]} extension.rs file(s) checked, no inline SQL."
