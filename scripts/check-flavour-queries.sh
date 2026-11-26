#!/usr/bin/env bash
# Gate: every compile-checked query a flavour builds names only relations that
# flavour's own migrations install.
#   scripts/check-flavour-queries.sh FLAVOUR|all
# Needs a reachable Postgres whose role may create databases, named by
# FLAVOUR_QUERIES_ADMIN_URL (default postgres://postgres:postgres@localhost:5432/postgres).
# For each flavour: build its systemprompt binary, migrate a fresh database
# with it, warm the dependency graph offline, then re-check the template's own
# sqlx crates online against that database. A query whose table, view,
# function or column the flavour does not install is a compile error at the
# query, reported with its file and line.
# Why: sqlx macros expand once per compile and their database is not a cargo
# fingerprint input, so the offline pass caches core's crates (checked against
# their own committed caches) and touching the template crate roots makes only
# those re-expand against the flavour database.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
usage() { sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//' >&2; exit 2; }
target="${1:-}"
[ -n "$target" ] || usage
admin_url="${FLAVOUR_QUERIES_ADMIN_URL:-postgres://postgres:postgres@localhost:5432/postgres}"
root="$FLAVOUR_ROOT"
cd "$root"
if [ "$target" = all ]; then
    mapfile -t flavours < <(flavour_py list)
else
    flavour_require "$target"
    flavours=("$target")
fi

sqlx_crates=()
while IFS= read -r manifest; do
    sqlx_crates+=("$(dirname "$manifest")")
done < <(grep -lE '^sqlx' Cargo.toml extensions/*/Cargo.toml extensions/*/*/Cargo.toml)

failed=()
for flavour in "${flavours[@]}"; do
    features="$(flavour_py features "$flavour")"
    db="flavour_queries_${flavour//-/_}"
    db_url="${admin_url%/*}/$db"
    target_dir="${CARGO_TARGET_DIR:-$root/target/flavour-$flavour}"
    echo "==> $flavour: building the binary"
    SQLX_OFFLINE=true CARGO_TARGET_DIR="$target_dir" cargo build --bin systemprompt \
        --no-default-features --features "$features"
    echo "==> $flavour: migrating a fresh $db"
    psql "$admin_url" -v ON_ERROR_STOP=1 -q -c "DROP DATABASE IF EXISTS $db" -c "CREATE DATABASE $db"
    "$target_dir/debug/systemprompt" --database-url "$db_url" infra db migrate
    echo "==> $flavour: warming dependencies offline"
    SQLX_OFFLINE=true CARGO_TARGET_DIR="$target_dir" cargo check --keep-going \
        --no-default-features --features "$features" >/dev/null 2>&1 || true
    for dir in "${sqlx_crates[@]}"; do
        find "$dir/src" -maxdepth 1 \( -name lib.rs -o -name main.rs \) -exec touch {} +
    done
    echo "==> $flavour: checking the template's queries against $db"
    log="$target_dir/flavour-queries.log"
    if SQLX_OFFLINE=false DATABASE_URL="$db_url" CARGO_TARGET_DIR="$target_dir" cargo check \
        --keep-going --message-format short --no-default-features --features "$features" >"$log" 2>&1; then
        echo "OK: $flavour"
    else
        grep ': error' "$log" | sed 's/ at line [0-9]*$//' | sort -u >&2 || tail -40 "$log" >&2
        failed+=("$flavour")
    fi
done

if [ ${#failed[@]} -gt 0 ]; then
    echo "check-flavour-queries: failed for ${failed[*]}" >&2
    exit 1
fi
echo "OK: every flavour's queries read only what it installs"
