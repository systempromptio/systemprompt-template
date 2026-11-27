#!/usr/bin/env bash
# Install the Android SDK packages the app build needs, at the versions
# extensions/mobile/shared/toolchain.env pins: the platform and the build
# tools. With --ndk, also the NDK (the Rust library step needs it; the app
# build does not). Accepts the SDK licences. Needs an SDK with the
# command-line tools already in place (ANDROID_HOME); hosted Linux and macOS
# runners have one.
# Run by `just mobile-android-sdk` and by the Mobile workflow's Linux job.
set -euo pipefail
source "$(dirname "$0")/../../extensions/mobile/shared/env.sh"
[ -n "${ANDROID_HOME:-}" ] || { echo "android-sdk: no Android SDK found: set ANDROID_HOME" >&2; exit 1; }
sdkmanager="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
[ -x "$sdkmanager" ] || sdkmanager="$(command -v sdkmanager || true)"
[ -n "$sdkmanager" ] || { echo "android-sdk: no sdkmanager under $ANDROID_HOME/cmdline-tools/latest or on PATH" >&2; exit 1; }
packages=("platforms;android-$MOBILE_ANDROID_COMPILE_SDK" "build-tools;$MOBILE_ANDROID_BUILD_TOOLS")
if [ "${1:-}" = --ndk ]; then packages+=("ndk;$MOBILE_ANDROID_NDK"); fi
# Why: `yes` ends on a closed pipe once sdkmanager has read enough, which
# pipefail would report as this script's failure.
set +o pipefail
yes | "$sdkmanager" --sdk_root="$ANDROID_HOME" --licenses > /dev/null
yes | "$sdkmanager" --sdk_root="$ANDROID_HOME" "${packages[@]}" > /dev/null
status="${PIPESTATUS[1]}"
set -o pipefail
[ "$status" -eq 0 ] || { echo "android-sdk: sdkmanager failed installing ${packages[*]}" >&2; exit "$status"; }
echo "android-sdk: installed ${packages[*]} into $ANDROID_HOME"
