#!/usr/bin/env bash
# Build, sign, notarize, and stage the macOS bridge as the single-file DMG the
# homepage and the admin Bridge Setup page link to
# (storage/files/downloads/astound-bridge-macos.dmg).
#
# Must run on macOS: the build needs the Apple toolchain, the .app wrap uses
# sips/iconutil (bridge/scripts/make-mac-app.sh), and signing needs a
# "Developer ID Application" identity in the login keychain.
#
# The binary is universal (arm64 + x86_64 via lipo) so one DMG serves both
# Apple Silicon and Intel Macs — which is why the asset has no arch suffix.
#
# Notarization is REQUIRED by default: a Developer-ID-signed but un-notarized
# app downloaded from the web is still blocked by Gatekeeper, so shipping one
# would look identical to shipping nothing. Store credentials once with:
#
#     xcrun notarytool store-credentials astound-bridge \
#         --apple-id <apple-id> --team-id <team-id> \
#         --password <app-specific-password>
#
# and this script picks the profile up by name (override with NOTARY_PROFILE).
# SKIP_NOTARIZE=1 exists for local smoke builds only — the staged DMG is then
# not shippable and the script says so loudly.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$REPO_ROOT/bridge"
DOWNLOADS_DIR="$REPO_ROOT/storage/files/downloads"
ASSET="astound-bridge-macos.dmg"
NOTARY_PROFILE="${NOTARY_PROFILE:-astound-bridge}"
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
UNIVERSAL="universal-apple-darwin"

[ "$(uname -s)" = "Darwin" ] || {
    echo "ERROR: must run on macOS (codesign/notarytool/hdiutil)." >&2; exit 1; }

for t in "${TARGETS[@]}"; do
    rustup target list --installed | grep -qx "$t" || {
        echo "ERROR: rust target missing — run: rustup target add $t" >&2; exit 1; }
done

# ── Signing identity ──────────────────────────────────────────────────────────
# Auto-detect the Developer ID cert; CODESIGN_IDENTITY overrides when the
# keychain holds more than one.
if [ -z "${CODESIGN_IDENTITY:-}" ]; then
    CODESIGN_IDENTITY="$(security find-identity -v -p codesigning \
        | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -1)"
    [ -n "$CODESIGN_IDENTITY" ] || {
        echo "ERROR: no 'Developer ID Application' identity in the keychain and" >&2
        echo "CODESIGN_IDENTITY unset. Install the cert or set the variable." >&2
        exit 1
    }
fi

# ── Notary credentials — checked BEFORE the (slow) build, not after ───────────
if [ "${SKIP_NOTARIZE:-0}" != "1" ]; then
    xcrun notarytool history --keychain-profile "$NOTARY_PROFILE" >/dev/null 2>&1 || {
        echo "ERROR: no notarytool credential profile '$NOTARY_PROFILE'." >&2
        echo "Store one (once) with:" >&2
        echo "    xcrun notarytool store-credentials $NOTARY_PROFILE \\" >&2
        echo "        --apple-id <apple-id> --team-id <team-id> --password <app-specific-password>" >&2
        echo "or set NOTARY_PROFILE, or SKIP_NOTARIZE=1 for a local-only build." >&2
        exit 1
    }
fi

# ── Build both slices and fuse them ───────────────────────────────────────────
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    for t in "${TARGETS[@]}"; do
        echo "==> building astound-bridge for $t"
        (cd "$BRIDGE_DIR" && cargo build --release --target "$t")
    done
fi
for t in "${TARGETS[@]}"; do
    [ -f "$BRIDGE_DIR/target/$t/release/astound-bridge" ] || {
        echo "ERROR: missing $t binary. Run without SKIP_BUILD=1." >&2; exit 1; }
done

UNI_DIR="$BRIDGE_DIR/target/$UNIVERSAL/release"
mkdir -p "$UNI_DIR"
lipo -create -output "$UNI_DIR/astound-bridge" \
    "$BRIDGE_DIR/target/aarch64-apple-darwin/release/astound-bridge" \
    "$BRIDGE_DIR/target/x86_64-apple-darwin/release/astound-bridge"
lipo -archs "$UNI_DIR/astound-bridge" | grep -q 'x86_64.*arm64\|arm64.*x86_64' || {
    echo "ERROR: lipo output is not universal:" >&2
    lipo -archs "$UNI_DIR/astound-bridge" >&2
    exit 1
}

# ── Wrap, sign, verify ────────────────────────────────────────────────────────
"$BRIDGE_DIR/scripts/make-mac-app.sh" --target "$UNIVERSAL"
APP="$UNI_DIR/AstoundBridge.app"

echo "==> signing with: $CODESIGN_IDENTITY"
# Hardened runtime is a notarization requirement. Inside-out: binary, then
# bundle. No entitlements needed — WKWebView JITs in its own XPC process.
codesign --force --options runtime --timestamp \
    --sign "$CODESIGN_IDENTITY" "$APP/Contents/MacOS/astound-bridge"
codesign --force --options runtime --timestamp \
    --sign "$CODESIGN_IDENTITY" "$APP"
codesign --verify --strict --deep "$APP"

# ── DMG ───────────────────────────────────────────────────────────────────────
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
DMG="$BRIDGE_DIR/target/$UNIVERSAL/release/$ASSET"
rm -f "$DMG"
hdiutil create -volname "Astound Bridge" -srcfolder "$STAGE" \
    -format UDZO -ov -quiet "$DMG"
codesign --force --timestamp --sign "$CODESIGN_IDENTITY" "$DMG"

# ── Notarize + staple ─────────────────────────────────────────────────────────
if [ "${SKIP_NOTARIZE:-0}" = "1" ]; then
    # Deliberately NOT staged: the coordinator records success per source
    # fingerprint (env vars excluded), so staging here would let a later
    # `just deploy` on the same tree skip packaging and ship a DMG that
    # Gatekeeper blocks on every other Mac.
    echo "==> SKIP_NOTARIZE=1 — built and signed, but NOT staged for deploy:" >&2
    echo "    $DMG" >&2
    exit 0
fi
echo "==> notarizing (waits for Apple; typically 1-5 minutes)"
xcrun notarytool submit "$DMG" --keychain-profile "$NOTARY_PROFILE" --wait
xcrun stapler staple "$DMG"
# The end-to-end check: would Gatekeeper accept this exact download?
spctl -a -t open --context context:primary-signature "$DMG"

# ── Stage ─────────────────────────────────────────────────────────────────────
mkdir -p "$DOWNLOADS_DIR"
install -m 0644 "$DMG" "$DOWNLOADS_DIR/$ASSET"
# shasum on macOS, sha256sum on Linux CI — output format is identical.
(cd "$DOWNLOADS_DIR" && shasum -a 256 "$ASSET" > "$ASSET.sha256")

echo "==> staged $DOWNLOADS_DIR/$ASSET"
cat "$DOWNLOADS_DIR/$ASSET.sha256"
echo "Served from /files/downloads at runtime; cloud deploy bakes storage/ into the image."
