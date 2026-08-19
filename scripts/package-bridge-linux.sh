#!/usr/bin/env bash
# Package the branded bridge as a Linux release tarball.
#
# Produces dist/astound-bridge-linux-x86_64.tar.gz plus a .sha256, matching the
# asset name the admin Bridge Setup page links to. Keep the two in lockstep:
# extensions/web/admin/src/handlers/ssr/ssr_bridge_setup.rs (DOWNLOAD_BASE_URL)
# and storage/files/admin/templates/bridge-setup.hbs (asset filenames).
#
# Verify the result on a machine with no config using: just clean-client
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$REPO_ROOT/bridge"
DIST_DIR="$REPO_ROOT/dist"
BIN_NAME="astound-bridge"

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)  ASSET_ARCH="x86_64" ;;
    aarch64) ASSET_ARCH="aarch64" ;;
    *) echo "ERROR: unsupported host arch '$ARCH' — build on x86_64 or aarch64." >&2; exit 1 ;;
esac
ASSET="${BIN_NAME}-linux-${ASSET_ARCH}.tar.gz"
BIN="$BRIDGE_DIR/target/release/$BIN_NAME"

# ── Build ─────────────────────────────────────────────────────────────────────
# The bridge is a standalone workspace (GUI deps, own release cadence), so it is
# built directly rather than through the build coordinator, which keys on the
# main workspace's fingerprint.
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    echo "==> building $BIN_NAME (release)"
    (cd "$BRIDGE_DIR" && cargo build --release)
fi
[ -f "$BIN" ] || { echo "ERROR: $BIN missing. Run without SKIP_BUILD=1." >&2; exit 1; }

# ── Record runtime dependencies ───────────────────────────────────────────────
# The binary dynamically links libdbus-1 (keyring-core's secret-service store),
# libsystemd, libcap, and libgcrypt. A minimal host without them fails at exec
# with "error while loading shared libraries", before any of our error handling
# runs — so the tarball states them rather than leaving users to decode ldd.
echo "==> resolving dynamic dependencies"
SONAMES="$(ldd "$BIN" | awk '{print $1}' | grep -E '^lib' | sort -u | tr '\n' ' ')"
if ldd "$BIN" | grep -q "not found"; then
    echo "ERROR: the build host itself is missing libraries:" >&2
    ldd "$BIN" | grep "not found" >&2
    exit 1
fi

# ── Stage ─────────────────────────────────────────────────────────────────────
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
PKG="$STAGE/${BIN_NAME}-linux-${ASSET_ARCH}"
mkdir -p "$PKG"
install -m 0755 "$BIN" "$PKG/$BIN_NAME"

VERSION="$(cd "$BRIDGE_DIR" && cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[0].version')"
COMMIT="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

cat > "$PKG/INSTALL.md" <<EOF
# Astound Bridge for Linux

Version ${VERSION} (${COMMIT}, ${ASSET_ARCH})

## One-line install

Prefer the installer — it verifies the checksum, installs to the right place,
and writes the environment for you:

    curl -fsSL https://your-gateway/files/downloads/install.sh | sh -s -- \\
      --download-base https://your-gateway/files/downloads

The rest of this file is the manual equivalent.

## Runtime dependencies

This binary links the following shared libraries:

    ${SONAMES}

On a minimal host, install them first or the binary will not start at all —
it fails at exec with \`error while loading shared libraries\`, before it can
print a diagnostic:

    # Debian / Ubuntu
    sudo apt-get install -y libdbus-1-3 libcap2 libgcrypt20 libsystemd0

    # RHEL / Fedora
    sudo dnf install -y dbus-libs libcap libgcrypt systemd-libs

\`libdbus-1\` is required because credentials can be stored via the freedesktop
Secret Service. Where no Secret Service provider is running, the bridge tiers
down to the kernel keyutils keyring and then to process memory, and says which
one it chose in \`${BIN_NAME} doctor\`.

## Install

    install -Dm755 ${BIN_NAME} ~/.local/bin/${BIN_NAME}

## Set up

    ${BIN_NAME} login sp-live-...   --gateway https://your-gateway
    ${BIN_NAME} install --apply --apply-schedule
    ${BIN_NAME} sync                # pull plugins, skills, agents
    ${BIN_NAME} doctor              # confirm

\`install --apply\` writes \`~/.config/astound/env.sh\` (ANTHROPIC_BASE_URL and
ANTHROPIC_AUTH_TOKEN) and a managed block in \`~/.profile\` that sources it.
\`--apply-schedule\` registers two systemd user units: the periodic sync timer
and \`${BIN_NAME}-proxy.service\`, which keeps the loopback inference proxy
running. Where systemd is unavailable the units are still written and the
command warns rather than fails; run the proxy by hand with \`${BIN_NAME} proxy\`.

Open a new login shell and \`claude\` works with no manual exports.

## Headless credentials

Device certificates are the supported unattended credential. Generate one,
have an admin enrol its fingerprint, then name it in the config:

    [mtls]
    cert_keystore_ref = "~/.config/astound/device.pem"

\`ASTOUND_BRIDGE_DEVICE_CERT\` still works and takes precedence.

## Uninstall

    ${BIN_NAME} uninstall           # units, env.sh, and the ~/.profile block
    rm ~/.local/bin/${BIN_NAME}
EOF

# ── Pack ──────────────────────────────────────────────────────────────────────
mkdir -p "$DIST_DIR"
# --sort=name plus fixed mtime/owner keeps the archive byte-reproducible, so a
# rebuild of the same commit yields the same checksum.
tar --sort=name --owner=0 --group=0 --numeric-owner \
    --mtime="@$(git -C "$REPO_ROOT" log -1 --format=%ct 2>/dev/null || echo 0)" \
    -czf "$DIST_DIR/$ASSET" -C "$STAGE" "$(basename "$PKG")"

(cd "$DIST_DIR" && sha256sum "$ASSET" > "$ASSET.sha256")

echo
echo "==> $DIST_DIR/$ASSET"
echo "    $(cd "$DIST_DIR" && cut -d' ' -f1 "$ASSET.sha256")"
echo "    version ${VERSION} (${COMMIT})  size $(du -h "$DIST_DIR/$ASSET" | cut -f1)"
# ── Publish the installer next to the tarball ─────────────────────────────────
# The admin Bridge Setup page serves both from storage/files/downloads/, so the
# installer's default download base and the tarball it fetches stay same-origin.
# INSTALL_BASE_URL bakes that origin into the published installer's
# DEFAULT_DOWNLOAD_BASE, so end users run it with no --download-base; override
# it when packaging for a different deployment target.
INSTALL_BASE_URL="${INSTALL_BASE_URL:-https://astound.systemprompt.io/files/downloads}"
PUBLISH_DIR="$REPO_ROOT/storage/files/downloads"
mkdir -p "$PUBLISH_DIR"
install -m 0644 "$DIST_DIR/$ASSET" "$PUBLISH_DIR/$ASSET"
install -m 0644 "$DIST_DIR/$ASSET.sha256" "$PUBLISH_DIR/$ASSET.sha256"
sed "s|@DOWNLOAD_BASE@|${INSTALL_BASE_URL%/}|" "$REPO_ROOT/scripts/install-bridge.sh" > "$PUBLISH_DIR/install.sh"
chmod 0644 "$PUBLISH_DIR/install.sh"
echo "==> published to $PUBLISH_DIR (tarball, .sha256, install.sh @ ${INSTALL_BASE_URL%/})"

echo
echo "Publish so the admin Bridge Setup links resolve:"
echo "    gh release create v${VERSION} $DIST_DIR/$ASSET $DIST_DIR/$ASSET.sha256"
