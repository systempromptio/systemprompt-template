#!/usr/bin/env bash
# Stage a flavour's services tree: the shared services/ subtrees its
# flavour.yaml lists, then the flavour's own flavours/<name>/services/ overrides.
#   scripts/flavour-stage-services.sh FLAVOUR DEST
# Every subtree is copied: core refuses a catalog directory that is a symlink,
# so a change under services/ takes effect on the next stage (just setup).
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour="${1:?usage: flavour-stage-services.sh FLAVOUR DEST}"
dest="${2:?usage: flavour-stage-services.sh FLAVOUR DEST}"
flavour_require "$flavour"
overrides="$FLAVOUR_ROOT/flavours/$flavour/services"
rm -rf "$dest"
mkdir -p "$dest"
while IFS= read -r subtree; do
    [ -n "$subtree" ] || continue
    source_dir="$FLAVOUR_ROOT/services/$subtree"
    [ -d "$source_dir" ] || { echo "services/$subtree does not exist" >&2; exit 1; }
    cp -a "$source_dir" "$dest/$subtree"
done < <(flavour_get "$flavour" services)
if [ -d "$overrides" ]; then
    cp -a "$overrides/." "$dest/"
fi
echo "staged services for $flavour into $dest"
