#!/usr/bin/env bash
# Release smoke of an exact image: the `full` flavour smoke in release mode
# (scripts/flavour-smoke.sh), on the one-click compose topology by default.
#   scripts/smoke-image.sh IMAGE [VERSION] [UPGRADE_FROM]
set -euo pipefail
image="${1:?usage: smoke-image.sh IMAGE [VERSION] [UPGRADE_FROM]}"
root="$(cd "$(dirname "$0")/.." && pwd)"
args=(full --image "$image" --release
    --compose "${SMOKE_COMPOSE_FILE:-$root/deploy/compose/one-click.docker-compose.yml}"
    --service "${SMOKE_APP_SERVICE:-app}")
if [ -n "${2:-}" ]; then args+=(--version "$2"); fi
if [ -n "${3:-}" ]; then args+=(--upgrade-from "$3"); fi
exec bash "$root/scripts/flavour-smoke.sh" "${args[@]}"
