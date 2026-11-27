# Recipes for the phone apps (extensions/mobile): a Gradle project for Android
# and an Xcode project for iOS, neither part of a cargo workspace. Imported by
# the justfile; every recipe is named mobile-*. The pins are in
# extensions/mobile/shared/toolchain.env. The Android recipes need JDK 17 and
# an Android SDK (ANDROID_HOME, or the SDK in its default place); the iOS
# recipes need macOS with Xcode.

MOBILE_SHARED := "extensions/mobile/shared"
MOBILE_APK := "extensions/mobile/android/app/build/outputs/apk/debug/app-debug.apk"

# Print the mobile toolchain pins and what this machine has; fail on pin drift
mobile-toolchain:
    bash scripts/mobile/toolchain.sh

# Install the pinned Android platform and build tools with sdkmanager (--ndk adds the NDK)
mobile-android-sdk *FLAGS:
    bash scripts/mobile/android-sdk.sh {{FLAGS}}

# Build the Android debug APK
mobile-android:
    bash {{MOBILE_SHARED}}/build-android.sh assemble

# Android lint, ktlint, detekt and the JVM unit tests (warnings are errors)
mobile-android-check:
    bash {{MOBILE_SHARED}}/build-android.sh check

# Install the debug APK on the connected device or emulator and launch it
mobile-android-install: mobile-android
    #!/usr/bin/env bash
    set -euo pipefail
    source "{{MOBILE_SHARED}}/env.sh"
    adb="${ANDROID_HOME:+$ANDROID_HOME/platform-tools/adb}"
    [ -n "$adb" ] && [ -x "$adb" ] || adb="$(command -v adb || true)"
    [ -n "$adb" ] || { echo "mobile: adb not found (platform-tools under ANDROID_HOME, or on PATH)" >&2; exit 1; }
    [ "$("$adb" get-state 2> /dev/null || true)" = device ] || { echo "mobile: no device or emulator is connected, or more than one is (set ANDROID_SERIAL): \`$adb devices\` lists them" >&2; exit 1; }
    "$adb" install -r "{{MOBILE_APK}}"
    "$adb" shell am start -n "$MOBILE_APP_ID/.MainActivity"

# Build the iOS app for the simulator, unsigned (macOS only)
mobile-ios:
    bash {{MOBILE_SHARED}}/build-ios.sh build

# Install the pinned SwiftLint into extensions/mobile/ios/build/tools (macOS only)
mobile-ios-tools:
    bash scripts/mobile/ios-tools.sh

# SwiftLint and swift-format over the iOS sources (macOS only; warnings are errors)
mobile-ios-check:
    bash {{MOBILE_SHARED}}/build-ios.sh check
