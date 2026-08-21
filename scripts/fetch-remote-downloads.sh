#!/usr/bin/env bash
# Backfill storage/files/downloads/ from the deployed host.
#
# Why this exists: storage/files/downloads/ is gitignored, and `cloud deploy`
# bakes the whole storage/ tree into the image it ships. Each platform's bridge
# binary can only be BUILT on that platform's toolchain (the Windows exe via
# cargo-xwin on Linux, the DMG on a Mac), so any single machine is always
# missing the artifacts the others produced. Deploying from such a machine
# without this step silently replaces the live image with one that 404s the
# other platforms' download links.
#
# For every asset the site links, this script keeps whatever is already staged
# locally (a fresh local build wins) and downloads the rest from the currently
# deployed host, verifying the published sha256. It FAILS if an asset exists
# neither locally nor remotely, because deploying would break a live link —
# FETCH_ALLOW_MISSING=1 overrides that for bootstrap deploys of a brand-new
# target that has nothing published yet.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOWNLOADS_DIR="$REPO_ROOT/storage/files/downloads"
BASE="${DOWNLOADS_REMOTE_BASE:-https://astound.systemprompt.io/files/downloads}"
BASE="${BASE%/}"

# Every asset a page links. Checksummed ones are listed bare — their .sha256
# sidecar is fetched and verified alongside. install.sh has no sidecar.
CHECKSUMMED=(
    astound-bridge-windows.exe
    astound-bridge-linux-x86_64.tar.gz
    astound-bridge-macos.dmg
)
PLAIN=(
    install.sh
)
# Linked from the Bridge Setup page but never yet published (the link 404s on
# the live host today). Preserved when present, skipped without error when
# absent — so publishing one later doesn't require touching this script.
OPTIONAL_CHECKSUMMED=(
    astound-bridge-linux-aarch64.tar.gz
)

if command -v sha256sum >/dev/null 2>&1; then
    SHA_CHECK=(sha256sum -c)
else
    SHA_CHECK=(shasum -a 256 -c)
fi

mkdir -p "$DOWNLOADS_DIR"
missing=()

fetch() { # fetch <name> -> 0 fetched, 1 not on remote
    local name="$1" tmp
    tmp="$(mktemp)"
    if curl -fsSL -o "$tmp" "$BASE/$name" 2>/dev/null; then
        mv "$tmp" "$DOWNLOADS_DIR/$name"
        chmod 0644 "$DOWNLOADS_DIR/$name"
        return 0
    fi
    rm -f "$tmp"
    return 1
}

for asset in "${CHECKSUMMED[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        # Local build wins; make sure its sidecar exists for the next machine.
        if [ ! -f "$DOWNLOADS_DIR/$asset.sha256" ]; then
            (cd "$DOWNLOADS_DIR" && { sha256sum "$asset" 2>/dev/null \
                || shasum -a 256 "$asset"; } > "$asset.sha256")
        fi
        echo "==> $asset: already staged locally, keeping it"
        continue
    fi
    if fetch "$asset" && fetch "$asset.sha256"; then
        (cd "$DOWNLOADS_DIR" && "${SHA_CHECK[@]}" "$asset.sha256" >/dev/null) || {
            echo "ERROR: checksum mismatch for fetched $asset — refusing to stage it." >&2
            rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
            exit 1
        }
        echo "==> $asset: fetched from $BASE and verified"
    else
        rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
        missing+=("$asset")
    fi
done

for asset in "${OPTIONAL_CHECKSUMMED[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        echo "==> $asset: already staged locally, keeping it"
    elif fetch "$asset" && fetch "$asset.sha256"; then
        (cd "$DOWNLOADS_DIR" && "${SHA_CHECK[@]}" "$asset.sha256" >/dev/null) || {
            echo "ERROR: checksum mismatch for fetched $asset — refusing to stage it." >&2
            rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
            exit 1
        }
        echo "==> $asset: fetched from $BASE and verified"
    else
        rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
        echo "==> $asset: not published anywhere yet, skipping (optional)"
    fi
done

for asset in "${PLAIN[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        echo "==> $asset: already staged locally, keeping it"
    elif fetch "$asset"; then
        echo "==> $asset: fetched from $BASE"
    else
        missing+=("$asset")
    fi
done

if [ "${#missing[@]}" -gt 0 ]; then
    echo >&2
    echo "Missing (not staged locally, not on $BASE):" >&2
    for m in "${missing[@]}"; do echo "    $m" >&2; done
    echo "Build them (just bridge-package-macos / bridge-package-windows /" >&2
    echo "bridge-package-linux) or, for a brand-new deployment target with" >&2
    echo "nothing published yet, re-run with FETCH_ALLOW_MISSING=1." >&2
    [ "${FETCH_ALLOW_MISSING:-0}" = "1" ] || exit 1
    echo "FETCH_ALLOW_MISSING=1 set — continuing without them." >&2
fi

echo "==> downloads complete: $(ls "$DOWNLOADS_DIR" | tr '\n' ' ')"
