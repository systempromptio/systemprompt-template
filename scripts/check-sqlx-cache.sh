#!/usr/bin/env bash
# Fail if any extension crate that uses a sqlx query!-family macro is not
# covered by an offline `.sqlx/` cache. Coverage is satisfied by either the
# crate's own `.sqlx/` directory or the workspace-root `.sqlx/` (lib crates are
# cached into the root by `cargo sqlx prepare --workspace`; binary/extension
# crates that the workspace prepare skips carry their own).
#
# This is a fast fail-early guard so a missing cache is named plainly instead of
# surfacing as an opaque "set DATABASE_URL or SQLX_OFFLINE" macro error inside a
# cold offline build. Per-query freshness is still enforced by the offline
# `cargo check --workspace --locked` step that runs after this.
#
# It also pins the shape `just prepare` depends on: `tests/.sqlx` must be a
# symlink to `../.sqlx` (the four query!-using test crates resolve their
# offline cache through it — a real directory there passes every other gate
# and breaks only the offline CI build of those crates), and no
# `.sqlx.prepare-parked` may be left behind (a prepare was killed between
# parking the tracked cache and restoring it; the next `just prepare` heals
# it, or `rm -rf .sqlx && mv .sqlx.prepare-parked .sqlx` by hand).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if [ -e .sqlx.prepare-parked ]; then
    echo "error: .sqlx.prepare-parked exists — a 'just prepare' was killed mid-pass." >&2
    echo "The tracked cache is parked there. Run 'just prepare' again (it recovers it" >&2
    echo "first) or restore by hand: rm -rf .sqlx && mv .sqlx.prepare-parked .sqlx" >&2
    exit 1
fi

if [ ! -L tests/.sqlx ] || [ "$(readlink tests/.sqlx)" != "../.sqlx" ]; then
    echo "error: tests/.sqlx must be a symlink to ../.sqlx (found: $(ls -ld tests/.sqlx 2>/dev/null || echo missing))" >&2
    echo "Restore it with: rm -rf tests/.sqlx && ln -s ../.sqlx tests/.sqlx" >&2
    exit 1
fi

# A query!-family macro invocation: sqlx::query!(), query_as!(), query_scalar!(),
# query_file!(), and the bare re-exported forms.
macro_re='(sqlx::)?query(_as|_scalar|_file|_file_as|_file_scalar|_with)?!'

root_cache_ok=false
if [ -d .sqlx ] && ls .sqlx/*.json >/dev/null 2>&1; then
    root_cache_ok=true
fi

missing=()
while IFS= read -r manifest; do
    crate_dir=$(dirname "$manifest")
    # Does this crate invoke a query macro in its own sources?
    if ! grep -rqE "$macro_re" "$crate_dir/src" --include='*.rs' 2>/dev/null; then
        continue
    fi
    # Covered by its own cache?
    if [ -d "$crate_dir/.sqlx" ] && ls "$crate_dir/.sqlx"/*.json >/dev/null 2>&1; then
        continue
    fi
    # Otherwise it must be covered by the workspace-root cache.
    if [ "$root_cache_ok" = true ]; then
        continue
    fi
    missing+=("$crate_dir")
done < <(find extensions -name Cargo.toml -not -path '*/target/*')

if [ ${#missing[@]} -gt 0 ]; then
    echo "error: extension crate(s) use sqlx query macros but have no .sqlx offline cache:" >&2
    printf '  - %s\n' "${missing[@]}" >&2
    echo >&2
    echo "Run 'just prepare' against a live database and commit the generated .sqlx/." >&2
    exit 1
fi

echo "sqlx offline cache present for all extension crates using query macros."
