#!/usr/bin/env bash
# Compatibility bridge — the canonical installer is install-gateway.sh.
#
# The Cloudflare Bulk Redirect for get.systemprompt.io still points at this path.
# Rather than break every `curl -sSL https://get.systemprompt.io | sh` in the
# wild, this shim forwards to install-gateway.sh with whatever flags were passed.
#
# To retire this shim: update the Cloudflare redirect target to
# scripts/install-gateway.sh, then delete this file.

set -euo pipefail

TARGET_URL="https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/scripts/install-gateway.sh"

if ! command -v curl >/dev/null 2>&1; then
  printf '[install] curl is required\n' >&2
  exit 1
fi

exec bash -c "$(curl -fsSL "$TARGET_URL")" -- "$@"
