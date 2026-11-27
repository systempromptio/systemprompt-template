#!/usr/bin/env bash
# Print the toolchain the mobile apps build with, and fail when a pin has
# drifted: extensions/mobile/shared/toolchain.env is repeated in the Gradle
# wrapper, the version catalog, the binding crate's manifest, the Xcode
# project and the Mobile workflow, and this is the check that they all say
# the same. It also holds the binding crate to the core pin: its lockfile
# must resolve core from the rev in bridge/CORE_REF. It also prints what this
# machine has (JDK, Android SDK packages, Rust targets, cargo-ndk, Xcode),
# without failing on what is absent: a Linux machine has no Xcode and still
# passes.
# Run by `just mobile-toolchain` and by both jobs of the Mobile workflow.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
source "$root/extensions/mobile/shared/env.sh"
cd "$root"

echo "pins (extensions/mobile/shared/toolchain.env)"
grep -E '^MOBILE_[A-Z_]+=' extensions/mobile/shared/toolchain.env | sed 's/^/  /'

drift=0
# expect FILE LABEL PINNED PATTERN: PATTERN is a sed expression printing the
# value FILE holds.
expect() {
    local found
    found="$(sed -n "$4" "$1" | head -1)"
    if [ "$found" != "$3" ]; then
        echo "toolchain: $2 is '$found' in $1, toolchain.env pins '$3'" >&2
        drift=1
    fi
}
catalog=extensions/mobile/android/gradle/libs.versions.toml
wrapper=extensions/mobile/android/gradle/wrapper/gradle-wrapper.properties
project=extensions/mobile/ios/Mobile.xcodeproj/project.pbxproj
workflow=.github/workflows/mobile.yml
binding=extensions/mobile/core/Cargo.toml
expect "$wrapper" "the Gradle version" "$MOBILE_GRADLE" 's|^distributionUrl=.*/gradle-\(.*\)-bin\.zip$|\1|p'
expect "$catalog" "agp" "$MOBILE_AGP" 's|^agp = "\(.*\)"$|\1|p'
expect "$catalog" "kotlin" "$MOBILE_KOTLIN" 's|^kotlin = "\(.*\)"$|\1|p'
expect "$catalog" "compileSdk" "$MOBILE_ANDROID_COMPILE_SDK" 's|^compileSdk = "\(.*\)"$|\1|p'
expect "$catalog" "targetSdk" "$MOBILE_ANDROID_TARGET_SDK" 's|^targetSdk = "\(.*\)"$|\1|p'
expect "$catalog" "minSdk" "$MOBILE_ANDROID_MIN_SDK" 's|^minSdk = "\(.*\)"$|\1|p'
expect "$catalog" "buildTools" "$MOBILE_ANDROID_BUILD_TOOLS" 's|^buildTools = "\(.*\)"$|\1|p'
expect "$catalog" "ndk" "$MOBILE_ANDROID_NDK" 's|^ndk = "\(.*\)"$|\1|p'
expect "$catalog" "jna" "$MOBILE_JNA" 's|^jna = "\(.*\)"$|\1|p'
expect "$catalog" "kotlinxCoroutines" "$MOBILE_KOTLINX_COROUTINES" 's|^kotlinxCoroutines = "\(.*\)"$|\1|p'
expect "$binding" "uniffi" "$MOBILE_UNIFFI" 's|^uniffi = "\(.*\)"$|\1|p'
expect "$binding" "the binding crate's version" "$MOBILE_APP_VERSION" 's|^version = "\(.*\)"$|\1|p'
expect "$workflow" "cargo-ndk" "$MOBILE_CARGO_NDK" 's|^ *CARGO_NDK_VERSION: "\(.*\)"$|\1|p'
expect "$catalog" "appVersionName" "$MOBILE_APP_VERSION" 's|^appVersionName = "\(.*\)"$|\1|p'
expect extensions/mobile/android/app/build.gradle.kts "applicationId" "$MOBILE_APP_ID" 's|^ *applicationId = "\(.*\)"$|\1|p'
expect "$project" "the bundle identifier" "$MOBILE_APP_ID" 's|^[[:space:]]*PRODUCT_BUNDLE_IDENTIFIER = \(.*\);$|\1|p'
expect "$project" "the iOS deployment target" "$MOBILE_IOS_MIN" 's|^[[:space:]]*IPHONEOS_DEPLOYMENT_TARGET = \(.*\);$|\1|p'
expect "$project" "the Swift version" "$MOBILE_SWIFT" 's|^[[:space:]]*SWIFT_VERSION = \(.*\);$|\1|p'
expect "$project" "the marketing version" "$MOBILE_APP_VERSION" 's|^[[:space:]]*MARKETING_VERSION = \(.*\);$|\1|p'
expect "$workflow" "the JDK" "$MOBILE_JDK" 's|^ *java-version: "\(.*\)"$|\1|p'

core_ref="$(tr -d '[:space:]' < bridge/CORE_REF)"
if ! grep -qF "systemprompt-core?rev=$core_ref#" extensions/mobile/core/Cargo.lock; then
    echo "toolchain: extensions/mobile/core/Cargo.lock does not resolve core at bridge/CORE_REF ($core_ref); run scripts/core-pin.sh $core_ref" >&2
    drift=1
fi

echo "this machine"
if command -v java > /dev/null; then
    echo "  java: $(java -version 2>&1 | head -1)"
else
    echo "  java: not found (JDK $MOBILE_JDK is needed for the Android app)"
fi
if [ -n "${ANDROID_HOME:-}" ]; then
    echo "  ANDROID_HOME: $ANDROID_HOME"
    for package in "platforms/android-$MOBILE_ANDROID_COMPILE_SDK" "build-tools/$MOBILE_ANDROID_BUILD_TOOLS" "ndk/$MOBILE_ANDROID_NDK"; do
        if [ -d "$ANDROID_HOME/$package" ]; then echo "  $package: present"; else echo "  $package: absent"; fi
    done
else
    echo "  ANDROID_HOME: not set and no SDK in the default place"
fi
if command -v rustup > /dev/null; then
    # Why: listing must not install the pinned toolchain on a machine that
    # only builds the iOS app.
    installed="$(RUSTUP_AUTO_INSTALL=0 rustup target list --installed 2> /dev/null || true)"
    for target in $(printf '%s' "$MOBILE_RUST_TARGETS_ANDROID" | tr ',' ' '); do
        if printf '%s\n' "$installed" | grep -qx "$target"; then echo "  rust target $target: present"; else echo "  rust target $target: absent (just mobile-rust-targets)"; fi
    done
    if command -v cargo-ndk > /dev/null; then echo "  cargo-ndk: present"; else echo "  cargo-ndk: absent (cargo install cargo-ndk --version $MOBILE_CARGO_NDK --locked)"; fi
else
    echo "  rust: rustup not found (the binding library needs the toolchain in rust-toolchain.toml)"
fi
if command -v xcodebuild > /dev/null; then
    echo "  xcode: $(xcodebuild -version 2> /dev/null | tr '\n' ' ')(minimum $MOBILE_XCODE_MIN)"
else
    echo "  xcode: not found (the iOS app builds only on macOS)"
fi

if [ "$drift" -ne 0 ]; then
    echo "toolchain: the pins disagree; fix the file named above or toolchain.env" >&2
    exit 1
fi
echo "toolchain: every pin agrees with toolchain.env"
