#!/usr/bin/env bash
# Check both workspaces against the MSRV they declare — and check that they
# declare the same one.
#
# The number is READ from the manifests, never written here. A hardcoded
# toolchain in this script would silently stop matching the day someone edits
# `rust-version`, which is the failure this whole gate exists to prevent: the
# CI job used to install 1.94 and then run the nightly from `rust-toolchain.toml`,
# passing while testing nothing.
#
# `RUSTUP_TOOLCHAIN` is what outranks a `rust-toolchain.toml`; installing a
# toolchain and calling bare `cargo` does not.
set -euo pipefail

cd "$(dirname "$0")/.."

read_msrv() {
    # rust-version from the [workspace.package] or [package] table.
    grep -m1 '^rust-version = ' "$1" | sed 's/.*"\(.*\)".*/\1/'
}

root_msrv=$(read_msrv Cargo.toml)
# Why: the bridge is a second, standalone workspace where it exists at all. The
# template has none, so its absence is normal rather than a misconfiguration.
bridge_msrv=""
[ -f bridge/Cargo.toml ] && bridge_msrv=$(read_msrv bridge/Cargo.toml)

if [ -z "$root_msrv" ]; then
    echo "error: no rust-version in Cargo.toml" >&2
    exit 1
fi
if [ -n "$bridge_msrv" ] && [ "$root_msrv" != "$bridge_msrv" ]; then
    echo "error: the two workspaces declare different MSRVs — they must agree:" >&2
    echo "  Cargo.toml:        $root_msrv" >&2
    echo "  bridge/Cargo.toml: $bridge_msrv" >&2
    exit 1
fi

clippy_msrv=$(grep -m1 '^msrv = ' clippy.toml | sed 's/.*"\(.*\)".*/\1/')
if [ "$clippy_msrv" != "$root_msrv" ]; then
    echo "error: clippy.toml msrv ($clippy_msrv) does not match Cargo.toml rust-version ($root_msrv)" >&2
    exit 1
fi

toolchain="${root_msrv}.0"
if ! rustup toolchain list | grep -q "^${toolchain}"; then
    echo "error: toolchain $toolchain missing — run: rustup toolchain install $toolchain" >&2
    exit 1
fi

# Why: assert the toolchain actually in use, not the one we asked for. This is
# the half that catches a `rust-toolchain.toml` override reinstating itself.
actual=$(RUSTUP_TOOLCHAIN="$toolchain" cargo --version)
case "$actual" in
    "cargo ${root_msrv}."*) ;;
    *)
        echo "error: expected cargo $root_msrv, got: $actual" >&2
        exit 1
        ;;
esac

echo "MSRV $root_msrv — checking both workspaces with $actual"
RUSTUP_TOOLCHAIN="$toolchain" SQLX_OFFLINE=true cargo check --workspace --quiet
if [ -n "$bridge_msrv" ]; then
    RUSTUP_TOOLCHAIN="$toolchain" SQLX_OFFLINE=true \
        cargo check --manifest-path bridge/Cargo.toml --workspace --quiet
fi
echo "MSRV OK on $root_msrv."
