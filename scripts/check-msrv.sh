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
# Why: the desktop shell is a separate workspace built by the same toolchain
# file, so its declared MSRV must be the root's. Only the number is compared:
# the workspace has no member to check until its first crate lands.
harness_msrv=$(read_msrv extensions/harness/Cargo.toml)
if [ "$root_msrv" != "$harness_msrv" ]; then
    echo "error: the root and shell workspaces declare different MSRVs — they must agree:" >&2
    echo "  Cargo.toml:                    $root_msrv" >&2
    echo "  extensions/harness/Cargo.toml: ${harness_msrv:-none}" >&2
    exit 1
fi
# Why: the phone apps' binding library is a separate workspace built by the
# same toolchain file, so its declared MSRV must be the root's. Only the
# number is compared: the crate is built for the phone targets by the Mobile
# workflow, not here.
mobile_msrv=$(read_msrv extensions/mobile/core/Cargo.toml)
if [ "$root_msrv" != "$mobile_msrv" ]; then
    echo "error: the root and mobile binding workspaces declare different MSRVs — they must agree:" >&2
    echo "  Cargo.toml:                        $root_msrv" >&2
    echo "  extensions/mobile/core/Cargo.toml: ${mobile_msrv:-none}" >&2
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
if ! rustup toolchain list | grep "^${toolchain}" >/dev/null; then
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
# Why: the root package refuses to compile without one flavour feature
# (src/lib.rs compile_error!), so it is checked with a flavour and the other
# members with their own defaults.
flavour_features="$(python3 scripts/flavour.py features "${FLAVOUR:-full}")"
RUSTUP_TOOLCHAIN="$toolchain" SQLX_OFFLINE=true \
    cargo check --workspace --exclude systemprompt-template --quiet
RUSTUP_TOOLCHAIN="$toolchain" SQLX_OFFLINE=true \
    cargo check -p systemprompt-template --no-default-features --features "$flavour_features" --quiet
if [ -n "$bridge_msrv" ]; then
    RUSTUP_TOOLCHAIN="$toolchain" SQLX_OFFLINE=true \
        cargo check --manifest-path bridge/Cargo.toml --workspace --quiet
fi
echo "MSRV OK on $root_msrv."
