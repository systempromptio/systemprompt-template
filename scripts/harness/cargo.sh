#!/usr/bin/env bash
# Run cargo over the desktop shell from inside extensions/harness. rustup picks
# the toolchain from the working directory, not from --manifest-path, so a
# cargo started at the repository root would build the shell with the
# template's nightly instead of the toolchain the workspace pins in its own
# rust-toolchain.toml. Run by the harness-* recipes, the pre-commit hook and
# the Harness workflow (Windows, macOS, Linux).
#
# The shell is two cargo workspaces: extensions/harness (the five crates) and
# extensions/harness/tests (the seven test crates, under the tests lint
# table). Every verb but build-release covers both, the shell first, and both
# build into one target directory (extensions/harness/target unless
# CARGO_TARGET_DIR says otherwise), so the tests reuse the shell's build.
#   scripts/harness/cargo.sh resolve        cargo metadata --locked (no compile)
#   scripts/harness/cargo.sh check          cargo check --workspace --all-targets --locked
#   scripts/harness/cargo.sh test           cargo test --workspace --locked
#   scripts/harness/cargo.sh clippy         cargo clippy --workspace --all-targets --locked -- -D warnings
#   scripts/harness/cargo.sh fmt-check      rustfmt --check under the template's toolchain
#   scripts/harness/cargo.sh build-release  cargo build --release --locked -p systemprompt-harness-shell
#   scripts/harness/cargo.sh deny           cargo deny check (the template's deny.toml)
# Anything after the verb is passed to cargo (for clippy, before the "--").
set -euo pipefail
usage="usage: scripts/harness/cargo.sh resolve|check|test|clippy|fmt-check|build-release|deny [cargo args]"
root="$(cd "$(dirname "$0")/../.." && pwd)"
shell="$root/extensions/harness"
cd "$shell"
# Why: cargo on a Windows runner is a native program and wants a drive-letter
# path; Git Bash's `pwd -W` prints one, and no other bash has the flag.
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd -W 2>/dev/null || pwd)/target}"
export CARGO_TARGET_DIR
verb="${1:?$usage}"
shift

# Run a command in the shell workspace, then in its tests workspace.
both() {
    local dir
    for dir in "$shell" "$shell/tests"; do
        echo "harness: ${dir#"$root"/}: $*"
        (cd "$dir" && "$@")
    done
}

case "$verb" in
    resolve)
        both cargo metadata --locked --format-version 1 "$@" > /dev/null
        echo "harness: extensions/harness and extensions/harness/tests resolve against their lockfiles"
        ;;
    check) both cargo check --workspace --all-targets --locked "$@" ;;
    test) both cargo test --workspace --locked "$@" ;;
    clippy) both cargo clippy --workspace --all-targets --locked "$@" -- -D warnings ;;
    fmt-check)
        # The template's rustfmt.toml uses options only its nightly accepts, so
        # formatting runs under the root toolchain file's channel.
        channel="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$root/rust-toolchain.toml")"
        [ -n "$channel" ] || { echo "harness: no channel in rust-toolchain.toml" >&2; exit 1; }
        RUSTUP_TOOLCHAIN="$channel" both cargo fmt --all "$@" -- --check
        ;;
    build-release) cargo build --release --locked -p systemprompt-harness-shell "$@" ;;
    deny) both cargo deny "$@" check ;;
    *)
        echo "$usage" >&2
        exit 2
        ;;
esac
