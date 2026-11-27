#!/usr/bin/env bash
# Install the linter the iOS check needs, at the version and checksum
# extensions/mobile/shared/toolchain.env pins: the portable SwiftLint binary,
# into extensions/mobile/ios/build/tools (ignored by git), where
# shared/build-ios.sh looks before PATH. swift-format ships with Xcode.
# Run by `just mobile-ios-tools` and by the Mobile workflow's macOS job.
set -euo pipefail
source "$(dirname "$0")/../../extensions/mobile/shared/env.sh"
[ "$(uname -s)" = Darwin ] || { echo "ios-tools: SwiftLint's portable binary is a macOS binary (this is $(uname -s))" >&2; exit 1; }
tools="$MOBILE_ROOT/ios/build/tools"
mkdir -p "$tools"
archive="$tools/portable_swiftlint.zip"
curl --fail --silent --show-error --location --output "$archive" \
    "https://github.com/realm/SwiftLint/releases/download/$MOBILE_SWIFTLINT/portable_swiftlint.zip"
actual="$(shasum -a 256 "$archive" | awk '{ print $1 }')"
if [ "$actual" != "$MOBILE_SWIFTLINT_SHA256" ]; then
    echo "ios-tools: portable_swiftlint.zip $MOBILE_SWIFTLINT has sha256 $actual, toolchain.env pins $MOBILE_SWIFTLINT_SHA256" >&2
    rm -f "$archive"
    exit 1
fi
unzip -o -q "$archive" swiftlint -d "$tools"
rm -f "$archive"
echo "ios-tools: installed SwiftLint $("$tools/swiftlint" version) into $tools"
