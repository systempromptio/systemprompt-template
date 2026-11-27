#!/usr/bin/env bash
# Run cargo over the desktop shell workspace from inside extensions/harness.
# rustup picks the toolchain from the working directory, not from
# --manifest-path, so a cargo started at the repository root would build the
# shell with the template's nightly instead of the toolchain the workspace
# pins in its own rust-toolchain.toml. Run by the harness-* recipes and by the
# Harness workflow (Windows, macOS, Linux).
#   scripts/harness/cargo.sh resolve     cargo metadata --locked (no compile)
#   scripts/harness/cargo.sh check       cargo check --workspace --all-targets --locked
#   scripts/harness/cargo.sh test        cargo test --workspace --locked
#   scripts/harness/cargo.sh clippy      cargo clippy --workspace --all-targets --locked
#   scripts/harness/cargo.sh fmt-check   rustfmt --check under the template's toolchain
# Anything after the verb is passed to cargo.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root/extensions/harness"
verb="${1:?usage: scripts/harness/cargo.sh resolve|check|test|clippy|fmt-check [cargo args]}"
shift
case "$verb" in
    resolve)
        cargo metadata --locked --format-version 1 "$@" > /dev/null
        echo "harness: extensions/harness/Cargo.toml resolves against its Cargo.lock"
        ;;
    check) cargo check --workspace --all-targets --locked "$@" ;;
    test) cargo test --workspace --locked "$@" ;;
    clippy) cargo clippy --workspace --all-targets --locked "$@" ;;
    fmt-check)
        # The template's rustfmt.toml uses options only its nightly accepts, so
        # formatting runs under the root toolchain file's channel.
        channel="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$root/rust-toolchain.toml")"
        [ -n "$channel" ] || { echo "harness: no channel in rust-toolchain.toml" >&2; exit 1; }
        RUSTUP_TOOLCHAIN="$channel" cargo fmt --all "$@" -- --check
        ;;
    *)
        echo "usage: scripts/harness/cargo.sh resolve|check|test|clippy|fmt-check [cargo args]" >&2
        exit 2
        ;;
esac
