#!/usr/bin/env bash
# systemprompt-gateway installer — https://get.systemprompt.io
#
# Installs the gateway SERVER binary. For the cowork client CLI, use
# scripts/install-cowork.sh or see docs/cowork/.
#
# Usage:   curl -sSL get.systemprompt.io | sh
#          curl -sSL get.systemprompt.io | sh -s -- --version v0.3.5
#          curl -sSL get.systemprompt.io | sh -s -- --prefix /usr/local --verify

set -euo pipefail

REPO="systempromptio/systemprompt-template"
BIN_NAME="systemprompt"
VERSION="latest"
PREFIX=""
VERIFY_COSIGN="false"

log()  { printf '\033[0;36m[install]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[0;33m[install]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[0;31m[install] error:\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --prefix)  PREFIX="$2";  shift 2 ;;
    --verify)  VERIFY_COSIGN="true"; shift ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) die "unknown flag: $1" ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"; }
need curl
need tar
need uname

uname_s=$(uname -s | tr '[:upper:]' '[:lower:]')
uname_m=$(uname -m)

case "$uname_s" in
  linux)  os="linux" ;;
  darwin) os="darwin" ;;
  msys*|mingw*|cygwin*)
    die "The gateway is Linux/macOS-only. On Windows install cowork instead: docs/cowork/scoop.md" ;;
  *) die "unsupported OS: $uname_s" ;;
esac

case "$uname_m" in
  x86_64|amd64) arch="amd64" ;;
  arm64|aarch64) arch="arm64" ;;
  *) die "unsupported arch: $uname_m" ;;
esac

target="${os}-${arch}"

if [ "$VERSION" = "latest" ]; then
  log "resolving latest gateway release..."
  # List releases and pick the first tag that does NOT start with cowork-v
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
    | grep -oE '"tag_name"\s*:\s*"[^"]+"' \
    | sed -E 's/.*"([^"]+)"$/\1/' \
    | grep -v '^cowork-' \
    | head -n1)
  [ -n "$VERSION" ] || die "could not resolve latest gateway release (v*, excluding cowork-v*)"
fi

# Strip leading 'v' for filename: v0.3.5 → 0.3.5
ver_noprefix="${VERSION#v}"

log "installing systemprompt-gateway ${VERSION} for ${target}"

tarball="systemprompt-gateway-${ver_noprefix}-${target}.tar.gz"
base="https://github.com/${REPO}/releases/download/${VERSION}"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

log "downloading ${tarball}..."
curl -fsSL "${base}/${tarball}" -o "${tmp}/${tarball}"
curl -fsSL "${base}/SHA256SUMS.gateway" -o "${tmp}/SHA256SUMS.gateway"

log "verifying SHA256..."
(cd "$tmp" && grep " ${tarball}$" SHA256SUMS.gateway | sha256sum -c -)

if [ "$VERIFY_COSIGN" = "true" ]; then
  need cosign
  log "verifying cosign signature..."
  curl -fsSL "${base}/SHA256SUMS.gateway.sig" -o "${tmp}/SHA256SUMS.gateway.sig"
  curl -fsSL "${base}/SHA256SUMS.gateway.pem" -o "${tmp}/SHA256SUMS.gateway.pem"
  cosign verify-blob \
    --certificate-identity-regexp="https://github.com/systempromptio/systemprompt-deploy/" \
    --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
    --signature "${tmp}/SHA256SUMS.gateway.sig" \
    --certificate "${tmp}/SHA256SUMS.gateway.pem" \
    "${tmp}/SHA256SUMS.gateway"
fi

log "extracting..."
tar -xzf "${tmp}/${tarball}" -C "$tmp"

if [ -z "$PREFIX" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr/local"
  else
    PREFIX="${HOME}/.local"
  fi
fi

dest="${PREFIX}/bin"
mkdir -p "$dest"

stage_dir="${tmp}/systemprompt-gateway-${ver_noprefix}-${target}"

installed=""
for b in systemprompt systemprompt-mcp-agent systemprompt-mcp-marketplace; do
  if [ -f "${stage_dir}/${b}" ]; then
    install -m 0755 "${stage_dir}/${b}" "${dest}/${b}"
    installed="${installed} ${b}"
  fi
done

[ -n "$installed" ] || die "no binaries found in tarball (stage dir: ${stage_dir})"

log "installed:${installed}"
log "location: ${dest}"

case ":$PATH:" in
  *":${dest}:"*) ;;
  *) warn "add ${dest} to your PATH: export PATH=\"${dest}:\$PATH\"" ;;
esac

log "verify with: ${BIN_NAME} --version"
