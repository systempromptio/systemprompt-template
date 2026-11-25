# shellcheck shell=bash
# Sourced by the flavour scripts: one way to reach scripts/flavour.py.
# Any python3 with PyYAML (python3-yaml) reads the flavour definitions.
FLAVOUR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
flavour_py() { python3 "$FLAVOUR_ROOT/scripts/flavour.py" "$@"; }
flavour_get() { flavour_py get "$1" "$2"; }
flavour_require() {
    flavour_py list | grep -qxF "$1" || { echo "unknown flavour '$1' (just flavours lists them)" >&2; exit 2; }
}

# Writes a compose override that runs IMAGE as SERVICE on an ephemeral
# loopback port and publishes nothing else, so a measuring or smoke stack
# never collides with a host service or another stack.
#   flavour_isolated_override OUT IMAGE SERVICE COMPOSE_ARGS...
# Why: compose merges port lists, so the base file's fixed host ports are
# dropped with the !override/!reset YAML tags, which JSON cannot carry.
flavour_isolated_override() {
    local out="$1" image="$2" service="$3" other
    shift 3
    {
        echo "services:"
        printf '  %s:\n    image: %s\n    ports: !override\n      - "127.0.0.1::8080"\n' "$service" "$image"
        while IFS= read -r other; do
            [ -n "$other" ] && [ "$other" != "$service" ] || continue
            printf '  %s:\n    ports: !reset []\n' "$other"
        done < <(docker compose "$@" config --format json | jq -r '.services | to_entries[] | select(.value.ports) | .key')
    } > "$out"
}
