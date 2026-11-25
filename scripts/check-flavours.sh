#!/usr/bin/env bash
# Gate: flavours/index.toml, the flavours/<name>/ directories, the root
# Cargo.toml [features] and the services/ subtrees each flavour ships agree.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour_py check
