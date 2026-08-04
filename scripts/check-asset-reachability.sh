#!/usr/bin/env bash
# Gate: every shipped frontend asset must be reachable from a template,
# script root, partial include, @font-face, or content reference.
# cargo-machete and the markup->asset gates only prove the forward direction;
# this proves the reverse, so deleted features cannot leave assets behind.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/check-asset-reachability.py
