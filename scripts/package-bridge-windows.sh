#!/usr/bin/env bash
# Cross-compile the branded bridge for Windows and stage it as the single-file
# download the homepage links to (storage/files/downloads/astound-bridge-windows.exe).
#
# Target is x86_64-pc-windows-msvc, NOT -gnu, and that is load-bearing:
# webview2-com-sys statically links WebView2LoaderStatic.lib only on msvc
# targets. A -gnu build dynamically imports WebView2Loader.dll, and since we
# ship a bare .exe with no DLL beside it, that binary dies at process start
# with "WebView2Loader.dll was not found" before main() runs. The CRT is
# static too (+crt-static) so the exe does not require the VC++ redistributable.
#
# Toolchain (all Linux-hosted, no Windows box involved):
#   rustup target add x86_64-pc-windows-msvc
#   cargo-xwin            — fetches the Windows SDK/CRT and drives lld-link
#   llvm-rc (any version) — compiles the branded .rsrc; without it winresource
#                           degrades to a warning and the icon/version info is
#                           silently dropped, so this script refuses to run.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$REPO_ROOT/bridge"
DOWNLOADS_DIR="$REPO_ROOT/storage/files/downloads"
TARGET="x86_64-pc-windows-msvc"
ASSET="astound-bridge-windows.exe"
BIN="$BRIDGE_DIR/target/$TARGET/release/astound-bridge.exe"

command -v cargo-xwin >/dev/null || {
    echo "ERROR: cargo-xwin not installed (cargo install cargo-xwin, or a" >&2
    echo "prebuilt from https://github.com/rust-cross/cargo-xwin/releases)." >&2
    exit 1
}
rustup target list --installed | grep -qx "$TARGET" || {
    echo "ERROR: rust target missing — run: rustup target add $TARGET" >&2
    exit 1
}

if [ -z "${RC_PATH:-}" ]; then
    RC_PATH="$(command -v llvm-rc || true)"
    [ -n "$RC_PATH" ] || RC_PATH="$(ls /usr/bin/llvm-rc-* 2>/dev/null | sort -V | tail -1 || true)"
    [ -n "$RC_PATH" ] || {
        echo "ERROR: no llvm-rc found and RC_PATH unset — the Windows icon and" >&2
        echo "version resource cannot be compiled (apt install llvm)." >&2
        exit 1
    }
    export RC_PATH
fi

if [ "${SKIP_BUILD:-0}" != "1" ]; then
    echo "==> building astound-bridge for $TARGET (static CRT, rc=$RC_PATH)"
    (cd "$BRIDGE_DIR" && RUSTFLAGS="-C target-feature=+crt-static" \
        cargo xwin build --release --target "$TARGET")
fi
[ -f "$BIN" ] || { echo "ERROR: $BIN missing. Run without SKIP_BUILD=1." >&2; exit 1; }

# The two failure modes this script exists to prevent — refuse to ship either.
if strings "$BIN" | grep -q "WebView2Loader.dll"; then
    echo "ERROR: built exe dynamically imports WebView2Loader.dll — the loader" >&2
    echo "was not statically linked (wrong target?). Refusing to stage it." >&2
    exit 1
fi
if ! objdump -h "$BIN" | grep -qi '\.rsrc'; then
    echo "ERROR: built exe has no .rsrc section — the branded icon/version" >&2
    echo "resource was dropped (llvm-rc failure?). Refusing to stage it." >&2
    exit 1
fi

install -m 0644 "$BIN" "$DOWNLOADS_DIR/$ASSET"
(cd "$DOWNLOADS_DIR" && sha256sum "$ASSET" > "$ASSET.sha256")

echo "==> staged $DOWNLOADS_DIR/$ASSET"
cat "$DOWNLOADS_DIR/$ASSET.sha256"
echo "Run 'just publish' to copy it into web/dist/."
