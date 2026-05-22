#!/usr/bin/env bash
# bridge (client) installer — developer-workstation companion to the gateway.
#
# Installs the systemprompt-bridge binary from the latest bridge-v* release on
# systempromptio/systemprompt-template. For the gateway server, use
# install-gateway.sh.
#
# Usage:   curl -sSL https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/scripts/install-cowork.sh | sh
#          ... | sh -s -- --version bridge-v0.9.0 --prefix /usr/local

set -euo pipefail

REPO="systempromptio/systemprompt-template"
BIN_NAME="systemprompt-bridge"
VERSION="latest"
PREFIX=""

log()  { printf '\033[0;36m[bridge]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[0;33m[bridge]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[0;31m[bridge] error:\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --prefix)  PREFIX="$2";  shift 2 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) die "unknown flag: $1" ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"; }
need curl

uname_s=$(uname -s | tr '[:upper:]' '[:lower:]')
uname_m=$(uname -m)

case "$uname_s" in
  linux)
    case "$uname_m" in
      x86_64|amd64)    asset_pattern='systemprompt-bridge-x86_64-unknown-linux-gnu' ;;
      *) die "unsupported Linux arch: $uname_m (only x86_64 is published)" ;;
    esac
    ext="" ;;
  darwin)
    case "$uname_m" in
      arm64|aarch64) asset_pattern='systemprompt-bridge-aarch64-apple-darwin' ;;
      *) die "unsupported macOS arch: $uname_m (only Apple Silicon is published)" ;;
    esac
    ext="" ;;
  msys*|mingw*|cygwin*)
    die "On Windows use Scoop: scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket && scoop install bridge" ;;
  *) die "unsupported OS: $uname_s" ;;
esac

if [ "$VERSION" = "latest" ]; then
  log "resolving latest bridge-v* release..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
    | grep -oE '"tag_name"\s*:\s*"bridge-v[^"]+"' \
    | head -n1 \
    | sed -E 's/.*"([^"]+)"$/\1/')
  [ -n "$VERSION" ] || die "could not resolve latest bridge-v* release"
fi

log "installing ${BIN_NAME} ${VERSION} (asset: ${asset_pattern}${ext})"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

base="https://github.com/${REPO}/releases/download/${VERSION}"

# bridge assets are raw binaries, not tarballs.
log "downloading ${asset_pattern}${ext}..."
if ! curl -fsSL "${base}/${asset_pattern}${ext}" -o "${tmp}/${BIN_NAME}"; then
  die "asset not found: ${base}/${asset_pattern}${ext}"
fi
chmod +x "${tmp}/${BIN_NAME}"

if [ -z "$PREFIX" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr/local"
  else
    PREFIX="${HOME}/.local"
  fi
fi

dest="${PREFIX}/bin"
mkdir -p "$dest"
install -m 0755 "${tmp}/${BIN_NAME}" "${dest}/${BIN_NAME}"

log "installed ${BIN_NAME} to ${dest}/${BIN_NAME}"

case ":$PATH:" in
  *":${dest}:"*) ;;
  *) warn "add ${dest} to your PATH: export PATH=\"${dest}:\$PATH\"" ;;
esac

log "verify: ${BIN_NAME} --version"
log "configure: ${BIN_NAME} config set gateway.url https://your-gateway.example.com"
