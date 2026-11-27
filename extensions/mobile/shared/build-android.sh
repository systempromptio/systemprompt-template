#!/usr/bin/env bash
# Build the Android app.
#   build-android.sh [assemble|check]
# assemble (the default): the Rust library step, then `gradlew assembleDebug`;
#   prints the debug APK's path and size. Unsigned beyond the debug key.
# check: `gradlew check`: Android lint, ktlint, detekt and the JVM unit tests,
#   every warning an error.
# The Rust library step is a stub (shared/env.sh); the app build is real.
# Needs JDK $MOBILE_JDK and an Android SDK with the platform and build tools
# toolchain.env names (scripts/mobile/android-sdk.sh installs them).
set -euo pipefail
source "$(dirname "$0")/env.sh"
mode="${1:-assemble}"
[ -n "${ANDROID_HOME:-}" ] || { echo "mobile: no Android SDK found: set ANDROID_HOME" >&2; exit 1; }
cd "$MOBILE_ROOT/android"
case "$mode" in
    assemble)
        mobile_rust_library Android "$MOBILE_RUST_TARGETS_ANDROID"
        ./gradlew --console=plain assembleDebug
        apk="$MOBILE_ROOT/android/app/build/outputs/apk/debug/app-debug.apk"
        [ -f "$apk" ] || { echo "mobile: the build left no APK at $apk" >&2; exit 1; }
        echo "mobile: $apk ($(wc -c < "$apk" | tr -d ' ') bytes)"
        ;;
    check)
        ./gradlew --console=plain check
        ;;
    *)
        echo "usage: build-android.sh [assemble|check]" >&2
        exit 2
        ;;
esac
