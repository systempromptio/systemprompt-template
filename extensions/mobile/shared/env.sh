# shellcheck shell=bash
# Sourced by the mobile scripts: loads toolchain.env and finds the Android SDK.
# Sets MOBILE_ROOT (extensions/mobile), every MOBILE_* pin, and ANDROID_HOME
# when it is unset and the SDK sits in its default place for this system.
MOBILE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
set -a
# shellcheck source=toolchain.env
source "$MOBILE_ROOT/shared/toolchain.env"
set +a

if [ -z "${ANDROID_HOME:-}" ]; then
    for candidate in "${ANDROID_SDK_ROOT:-}" "$HOME/Android/Sdk" "$HOME/Library/Android/sdk"; do
        if [ -n "$candidate" ] && [ -d "$candidate" ]; then
            export ANDROID_HOME="$candidate"
            break
        fi
    done
fi

# The Rust library step both build scripts start with. A stub: the binding
# library is built from the systemprompt-mobile crate, which does not exist
# yet, so there is nothing to compile or generate.
#   mobile_rust_library PLATFORM TARGETS
mobile_rust_library() {
    echo "mobile: the Rust binding library for $1 ($2) arrives with the systemprompt-mobile crate; nothing to build yet"
}
