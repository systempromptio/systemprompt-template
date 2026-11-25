# shellcheck shell=bash
# Sourced by the flavour scripts: one way to reach scripts/flavour.py.
# Any python3 with PyYAML (python3-yaml) reads the flavour definitions.
FLAVOUR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
flavour_py() { python3 "$FLAVOUR_ROOT/scripts/flavour.py" "$@"; }
flavour_get() { flavour_py get "$1" "$2"; }
flavour_require() {
    flavour_py list | grep -qxF "$1" || { echo "unknown flavour '$1' (just flavours lists them)" >&2; exit 2; }
}
