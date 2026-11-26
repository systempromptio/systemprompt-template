#!/usr/bin/env bash
# Apply flavours/profile.base.yaml, then a flavour's profile.overlay.yaml (plus KEY=VALUE pairs the caller
# fixes, e.g. paths.services) to a profile. Keys an `admin config` command
# owns go through that command; the rest are written by a YAML merge because
# no command sets them (paths.*, and any key outside the table below).
#   scripts/flavour-overlay.sh FLAVOUR PROFILE BIN [KEY=VALUE...]
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour="${1:?usage: flavour-overlay.sh FLAVOUR PROFILE BIN [KEY=VALUE...]}"
profile="${2:?}"
bin="${3:?}"
shift 3
flavour_require "$flavour"
base="$FLAVOUR_ROOT/flavours/profile.base.yaml"
overlay="$FLAVOUR_ROOT/flavours/$flavour/profile.overlay.yaml"
pairs="$(python3 - "$base" "$overlay" "$@" <<'PY'
import sys, yaml
def flatten(prefix, value):
    if isinstance(value, dict):
        for key, inner in value.items():
            yield from flatten(f"{prefix}{key}.", inner)
    else:
        yield prefix[:-1], value
for source in sys.argv[1:3]:
    data = yaml.safe_load(open(source)) or {}
    for key, value in flatten("", data):
        print(f"{key}={yaml.safe_dump(value, default_flow_style=True).strip().removesuffix('...').strip()}")
for pair in sys.argv[3:]:
    print(pair)
PY
)"
merge=()
while IFS= read -r pair; do
    [ -n "$pair" ] || continue
    key="${pair%%=*}"
    value="${pair#*=}"
    case "$key" in
        server.host) "$bin" admin config server set --host "$value" ;;
        server.port) "$bin" admin config server set --port "$value" ;;
        server.api_external_url) "$bin" admin config server set --external-url "$value" ;;
        runtime.environment) "$bin" admin config runtime set --environment "$value" ;;
        runtime.log_level) "$bin" admin config runtime set --log-level "$value" ;;
        security.access_token_expiration) "$bin" admin config security set --access-expiry "$value" ;;
        security.refresh_token_expiration) "$bin" admin config security set --refresh-expiry "$value" ;;
        *) merge+=("$pair") ;;
    esac
done <<< "$pairs"
[ ${#merge[@]} -gt 0 ] || exit 0
python3 - "$profile" "${merge[@]}" <<'PY'
import os, sys, yaml
path = sys.argv[1]
profile = yaml.safe_load(open(path)) or {}
for pair in sys.argv[2:]:
    key, raw = pair.split("=", 1)
    node = profile
    *parents, leaf = key.split(".")
    for part in parents:
        node = node.setdefault(part, {})
    node[leaf] = yaml.safe_load(raw) if raw else raw
temporary = path + ".tmp"
with open(temporary, "w") as handle:
    yaml.safe_dump(profile, handle, sort_keys=False)
os.chmod(temporary, 0o600)
os.replace(temporary, path)
PY
