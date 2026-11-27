# shellcheck shell=bash
# Sourced by the flavour scripts: one way to reach scripts/flavour.py.
# Any python3 with PyYAML (python3-yaml) reads the flavour definitions.
FLAVOUR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
flavour_py() { python3 "$FLAVOUR_ROOT/scripts/flavour.py" "$@"; }
flavour_get() { flavour_py get "$1" "$2"; }
flavour_require() {
    flavour_py list | grep -xF "$1" >/dev/null || { echo "unknown flavour '$1' (just flavours lists them)" >&2; exit 2; }
}

# The scripts that build, set up, smoke or measure a server image call this
# first. For an image flavour it returns 1 and the script carries on; for any
# other artefact it prints where that artefact is built and returns 0, and the
# caller exits 0.
#   if flavour_not_image FLAVOUR "build"; then exit 0; fi
flavour_not_image() {
    local artefact
    artefact="$(flavour_get "$1" artefact)"
    case "$artefact" in
        image) return 1 ;;
        desktop) echo "flavour $1 is a desktop artefact, not an image: the desktop build arrives in stage S1 of the harness programme, so there is nothing to $2 yet" ;;
        mobile) echo "flavour $1 is a mobile artefact, not an image: the phone apps are built from $(flavour_get "$1" workspace) by \`just mobile-android\` and \`just mobile-ios\`, so this script has nothing to $2" ;;
        *) echo "flavour $1 is a $artefact artefact, not an image, so this script has nothing to $2" ;;
    esac
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
