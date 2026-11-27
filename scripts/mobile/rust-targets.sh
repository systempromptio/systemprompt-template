#!/usr/bin/env bash
# Add the Rust targets the binding library (extensions/mobile/core) is built
# for, as extensions/mobile/shared/toolchain.env pins them, to the toolchain
# rust-toolchain.toml names.
#   rust-targets.sh android|ios
# Run by `just mobile-rust-targets` and by the Mobile workflow's Linux job.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
source "$root/extensions/mobile/shared/env.sh"
case "${1:-}" in
    android) targets="$MOBILE_RUST_TARGETS_ANDROID" ;;
    ios) targets="$MOBILE_RUST_TARGETS_IOS" ;;
    *)
        echo "usage: rust-targets.sh android|ios" >&2
        exit 2
        ;;
esac
# Why: run from the repository, so rustup adds them to the pinned toolchain
# and not to whichever one is the machine's default.
cd "$root"
# shellcheck disable=SC2046
rustup target add $(printf '%s' "$targets" | tr ',' ' ')
