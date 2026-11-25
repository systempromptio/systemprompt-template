#!/usr/bin/env bash
# Stage a flavour's services tree: the shared services/ subtrees its
# flavour.yaml lists, then the flavour's own flavours/<name>/services/ overrides.
#   scripts/flavour-stage-services.sh FLAVOUR DEST [--link]
# --link (local setup) symlinks each shared subtree a flavour does not override,
# so edits under services/ stay live; without it (Docker) everything is copied.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour="${1:?usage: flavour-stage-services.sh FLAVOUR DEST [--link]}"
dest="${2:?usage: flavour-stage-services.sh FLAVOUR DEST [--link]}"
mode="${3:-copy}"
case "$mode" in copy|--link) ;; *) echo "unknown option: $mode" >&2; exit 2 ;; esac
flavour_require "$flavour"
overrides="$FLAVOUR_ROOT/flavours/$flavour/services"
rm -rf "$dest"
mkdir -p "$dest"
while IFS= read -r subtree; do
    [ -n "$subtree" ] || continue
    source_dir="$FLAVOUR_ROOT/services/$subtree"
    [ -d "$source_dir" ] || { echo "services/$subtree does not exist" >&2; exit 1; }
    if [ "$mode" = --link ] && [ ! -e "$overrides/$subtree" ]; then
        ln -s "$source_dir" "$dest/$subtree"
    else
        cp -a "$source_dir" "$dest/$subtree"
    fi
done < <(flavour_get "$flavour" services)
if [ -d "$overrides" ]; then
    cp -a "$overrides/." "$dest/"
fi
echo "staged services for $flavour into $dest"
