#!/usr/bin/env bash
# Gate: flavours/index.yaml, the flavours/<name>/ directories, the root
# Cargo.toml [features] and the services/ subtrees each image flavour ships
# agree; a desktop flavour names an existing cargo workspace and sets none of
# the keys an image reads.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour_py check
