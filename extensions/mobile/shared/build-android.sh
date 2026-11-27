#!/usr/bin/env bash
# Build the Android app.
#   build-android.sh [assemble|check|connected|library]
# Every mode starts with the Rust library step (shared/env.sh): the binding
# crate (core/) compiled with cargo-ndk for each Android ABI into the app's
# jniLibs, and the Kotlin that calls it generated into the app's build
# directory. Both are build output; the app does not compile without them.
# assemble (the default): then `gradlew assembleDebug`; prints the debug
#   APK's path and size. Unsigned beyond the debug key.
# check: rustfmt and clippy over the binding crate, then `gradlew check`:
#   Android lint, ktlint, detekt and the JVM unit tests. Every warning is an
#   error.
# connected: then `gradlew connectedDebugAndroidTest` on the connected device
#   or emulator (just mobile-android-emulator starts one). The discovery test
#   reads a stub instance on this machine (shared/stub-instance.py), which
#   this mode starts unless one is already listening on MOBILE_STUB_PORT.
# library: the Rust library step alone.
# MOBILE_RUST_PROFILE=release builds the library with the release profile
# (small and slow to build); the default is the dev profile.
# Needs JDK $MOBILE_JDK, an Android SDK with the platform, build tools and NDK
# toolchain.env names (scripts/mobile/android-sdk.sh --ndk installs them), the
# Rust targets it names (scripts/mobile/rust-targets.sh android) and
# cargo-ndk $MOBILE_CARGO_NDK.
set -euo pipefail
source "$(dirname "$0")/env.sh"
mode="${1:-assemble}"
case "$mode" in
    assemble | check | connected | library) ;;
    *)
        echo "usage: build-android.sh [assemble|check|connected|library]" >&2
        exit 2
        ;;
esac
[ -n "${ANDROID_HOME:-}" ] || mobile_fail "no Android SDK found: set ANDROID_HOME"
if [ "$mode" = check ]; then mobile_rust_lint; fi
mobile_rust_library_android
cd "$MOBILE_ROOT/android"
case "$mode" in
    assemble)
        ./gradlew --console=plain assembleDebug
        apk="$MOBILE_ROOT/android/app/build/outputs/apk/debug/app-debug.apk"
        [ -f "$apk" ] || mobile_fail "the build left no APK at $apk"
        echo "mobile: $apk ($(wc -c < "$apk" | tr -d ' ') bytes)"
        ;;
    check)
        ./gradlew --console=plain check
        ;;
    connected)
        stub="$MOBILE_ROOT/shared/stub-instance.py"
        if ! python3 "$stub" --probe --port "$MOBILE_STUB_PORT"; then
            python3 "$stub" --port "$MOBILE_STUB_PORT" &
            stub_pid=$!
            trap 'kill "$stub_pid" 2> /dev/null || true' EXIT
            sleep 1
            python3 "$stub" --probe --port "$MOBILE_STUB_PORT" \
                || mobile_fail "the stub instance did not start on port $MOBILE_STUB_PORT"
        fi
        # Why: the stub's port is forwarded over adb, so the device reads it
        # at its own loopback. That reaches an emulator with no guest network
        # (one running without hardware acceleration has none) and a device on
        # a cable alike; 10.0.2.2 reaches only an emulator whose network is up.
        # MOBILE_STUB_ADDRESS names another address when the stub is elsewhere.
        "$ANDROID_HOME/platform-tools/adb" reverse "tcp:$MOBILE_STUB_PORT" "tcp:$MOBILE_STUB_PORT" > /dev/null \
            || mobile_fail "adb could not forward port $MOBILE_STUB_PORT to the device"
        ./gradlew --console=plain connectedDebugAndroidTest \
            "-Pandroid.testInstrumentationRunnerArguments.stubInstance=${MOBILE_STUB_ADDRESS:-http://127.0.0.1:$MOBILE_STUB_PORT}"
        ;;
    library) ;;
esac
