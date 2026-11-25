# shellcheck shell=bash
# Sourced by the flavour scripts: one way to reach scripts/flavour.py.
# FLAVOUR_PYTHON names a Python 3.11+ (tomllib) when python3 is older.
FLAVOUR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
flavour_py() { "${FLAVOUR_PYTHON:-python3}" "$FLAVOUR_ROOT/scripts/flavour.py" "$@"; }
flavour_get() { flavour_py get "$1" "$2"; }
flavour_require() {
    flavour_py list | grep -qxF "$1" || { echo "unknown flavour '$1' (just flavours lists them)" >&2; exit 2; }
}
