#!/usr/bin/env bash
# Build and launch the Claude Code clean-room container against the gateway
# running on this host.
#
# Usage:
#   SP_BRIDGE_PAT=sp-live-... ./run.sh
#
# Env:
#   SP_BRIDGE_PAT          (required) PAT issued from /admin/devices or the
#                          admin API (POST /api/v1/admin/api-keys).
#   SP_BRIDGE_GATEWAY_URL  (default http://host.docker.internal:8080) gateway
#                          base URL as seen from inside the container. The
#                          container reaches the host gateway via
#                          host.docker.internal (works on Docker Desktop even
#                          when the gateway binds 127.0.0.1). On native Linux
#                          Docker, the --add-host below maps it to the host, but
#                          the gateway must then bind 0.0.0.0.
#   SP_BRIDGE_BIN          (default ../../../systemprompt-core/bin/bridge/target/
#                          release/systemprompt-bridge) host path to the Linux
#                          bridge binary (build with `just build-bridge` in core).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${HERE}/../.." && pwd)"

: "${SP_BRIDGE_PAT:?Set SP_BRIDGE_PAT (issue one from /admin/devices or POST /api/v1/admin/api-keys)}"
SP_BRIDGE_GATEWAY_URL="${SP_BRIDGE_GATEWAY_URL:-http://host.docker.internal:8080}"
SP_BRIDGE_BIN="${SP_BRIDGE_BIN:-${REPO_ROOT}/../systemprompt-core/bin/bridge/target/release/systemprompt-bridge}"

if [ ! -x "${SP_BRIDGE_BIN}" ]; then
  echo "Bridge binary not found at: ${SP_BRIDGE_BIN}" >&2
  echo "Build it first:  (cd ../systemprompt-core && just build-bridge)" >&2
  echo "Or set SP_BRIDGE_BIN to its path." >&2
  exit 1
fi

IMAGE="systemprompt/claude-code-clean-room:latest"

echo "==> Building ${IMAGE}"
docker build -t "${IMAGE}" "${HERE}"

echo "==> Running clean room (gateway=${SP_BRIDGE_GATEWAY_URL})"
exec docker run --rm -it \
  --add-host=host.docker.internal:host-gateway \
  -v "${SP_BRIDGE_BIN}:/usr/local/bin/systemprompt-bridge:ro" \
  -e SP_BRIDGE_PAT="${SP_BRIDGE_PAT}" \
  -e SP_BRIDGE_GATEWAY_URL="${SP_BRIDGE_GATEWAY_URL}" \
  "${IMAGE}"
