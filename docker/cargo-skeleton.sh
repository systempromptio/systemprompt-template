#!/usr/bin/env bash
# Copy a workspace's manifests into DEST with stub sources, so the image's
# dependency build is its own layer that a source-only change leaves cached.
#   docker/cargo-skeleton.sh SRC DEST
# Every Cargo.toml outside tests/ is copied; each lib.rs, main.rs and explicit
# target path it has in SRC becomes an empty stub (fn main for a binary).
# Build scripts are left out: they belong to the source build, not the deps.
set -euo pipefail
src="${1:?usage: cargo-skeleton.sh SRC DEST}"
dest="${2:?usage: cargo-skeleton.sh SRC DEST}"
cd "$src"
mkdir -p "$dest"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$dest/"
cp -a .cargo "$dest/.cargo"
[ -d vendor ] && cp -a vendor "$dest/vendor"
find . -name Cargo.toml -not -path './Cargo.toml' -not -path './tests/*' \
    -not -path './vendor/*' -not -path '*/target/*' -print0 |
while IFS= read -r -d '' manifest; do
    dir="$(dirname "$manifest")"
    mkdir -p "$dest/$dir"
    cp "$manifest" "$dest/$dir/Cargo.toml"
done
find . -name Cargo.toml -not -path './tests/*' -not -path './vendor/*' -not -path '*/target/*' -print0 |
while IFS= read -r -d '' manifest; do
    dir="$(dirname "$manifest")"
    targets="$(sed -n 's/^path *= *"\(.*\.rs\)"/\1/p' "$manifest")"
    for target in src/lib.rs src/main.rs $targets; do
        [ -f "$dir/$target" ] || continue
        mkdir -p "$(dirname "$dest/$dir/$target")"
        if grep -q 'fn main' "$dir/$target"; then
            echo 'fn main() {}' > "$dest/$dir/$target"
        else
            : > "$dest/$dir/$target"
        fi
    done
done
