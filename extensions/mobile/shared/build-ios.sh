#!/usr/bin/env bash
# Build the iOS app. macOS only: Xcode does not run anywhere else.
#   build-ios.sh [build|check]
# build (the default): `xcodebuild build` of the Mobile scheme for the
#   simulator, unsigned.
# check: SwiftLint (--strict) and swift-format (lint --strict) over the
#   sources, every warning an error.
# The iOS app does not link the binding library yet: its Rust library step
# (shared/env.sh) builds nothing and says so. The app build is real.
# Needs Xcode $MOBILE_XCODE_MIN or later.
set -euo pipefail
source "$(dirname "$0")/env.sh"
mode="${1:-build}"
if [ "$(uname -s)" != Darwin ]; then
    echo "mobile: the iOS app builds only on macOS (this is $(uname -s)); the Mobile workflow's macOS job builds it" >&2
    exit 1
fi
cd "$MOBILE_ROOT/ios"
case "$mode" in
    build)
        mobile_rust_library_ios
        xcodebuild \
            -project Mobile.xcodeproj \
            -scheme Mobile \
            -configuration Debug \
            -sdk iphonesimulator \
            -destination "generic/platform=iOS Simulator" \
            -derivedDataPath build/DerivedData \
            CODE_SIGNING_ALLOWED=NO \
            build
        echo "mobile: built $MOBILE_ROOT/ios/build/DerivedData/Build/Products/Debug-iphonesimulator/Mobile.app"
        ;;
    check)
        swiftlint="build/tools/swiftlint"
        [ -x "$swiftlint" ] || swiftlint="$(command -v swiftlint || true)"
        [ -n "$swiftlint" ] || { echo "mobile: SwiftLint is not installed (just mobile-ios-tools)" >&2; exit 1; }
        "$swiftlint" lint --strict --config .swiftlint.yml
        xcrun swift-format lint --strict --recursive --configuration .swift-format App Design Features Tests UITests
        ;;
    *)
        echo "usage: build-ios.sh [build|check]" >&2
        exit 2
        ;;
esac
