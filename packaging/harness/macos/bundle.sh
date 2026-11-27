#!/usr/bin/env bash
# bundle.sh: assembles the macOS app bundle of one (brand, channel) build of the harness.
#
#   dist/<Display name>.app                         the bundle, ad-hoc signed
#   dist/<scheme>-<version>-macos-<arch>.zip        the bundle, zipped with ditto (resource forks and signature kept)
#   dist/SHA256SUMS                                 the zip's SHA-256, in sha256sum format
#
# The identity table below is the packaging side of extensions/harness/brand/src/identity.rs (sp-brand): display name,
# app id (the bundle identifier) and URL scheme per brand and channel, as in packaging/harness/windows/package.ps1.
# The three change together; the version is read from sp-brand itself. The cargo build turns on the matching sp-brand
# features, so the binary and its bundle always carry the same identity, and MACOSX_DEPLOYMENT_TARGET matches
# LSMinimumSystemVersion.
#
# The bundle declares the build's URL scheme (CFBundleURLTypes), so LaunchServices routes <scheme>:// links to it. The
# signature is ad hoc (codesign -s -): enough for Apple silicon to run it and for the login keychain to tell builds
# apart, not a Developer ID; nothing is notarised. Without an icon in extensions/harness/shell/assets/icons the
# bundle has none and a warning says so.
#
# Usage:
#   packaging/harness/macos/bundle.sh [--brand systemprompt] [--channel internal|canary|dev] [--version X.Y.Z]
#                                     [--skip-build] [--dry-run]
#
# --dry-run builds nothing and needs no binary: it prints the bundle it would assemble and lints its Info.plist.
set -euo pipefail

MIN_MACOS="11.0"
EXE="harness"

brand="systemprompt"
channel="internal"
version=""
skip_build=0
dry_run=0

usage() {
    sed -n '/^# Usage:/,/^# --dry-run/p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --brand) [ "$#" -ge 2 ] || usage; brand="$2"; shift 2 ;;
        --channel) [ "$#" -ge 2 ] || usage; channel="$2"; shift 2 ;;
        --version) [ "$#" -ge 2 ] || usage; version="$2"; shift 2 ;;
        --skip-build) skip_build=1; shift ;;
        --dry-run) dry_run=1; shift ;;
        -h | --help) usage ;;
        *) echo "bundle.sh: unknown argument '$1'" >&2; usage ;;
    esac
done

case "$brand" in
    systemprompt) product="systemprompt Harness"; app_id="io.systemprompt.harness"; scheme="sp-harness"; brand_feature="" ;;
    *) echo "bundle.sh: brand '$brand' is not systemprompt" >&2; exit 2 ;;
esac
case "$channel" in
    internal) display_suffix=""; id_suffix=""; scheme_suffix=""; channel_feature="" ;;
    canary) display_suffix=" Canary"; id_suffix=".canary"; scheme_suffix="-canary"; channel_feature="channel-canary" ;;
    dev) display_suffix=" Dev"; id_suffix=".dev"; scheme_suffix="-dev"; channel_feature="channel-dev" ;;
    *) echo "bundle.sh: channel '$channel' is not internal, canary or dev" >&2; exit 2 ;;
esac
display_name="$product$display_suffix"
app_id="$app_id$id_suffix"
scheme="$scheme$scheme_suffix"

root="$(cd "$(dirname "$0")/../../.." && pwd)"
workspace="$root/extensions/harness"
dist="$root/dist"
release="$workspace/target/release"
icon_dir="$workspace/shell/assets/icons"
brand_source="$workspace/brand/src/lib.rs"

if [ -z "$version" ]; then
    version="$(sed -nE 's/^const VERSION: &str = "([^"]+)";$/\1/p' "$brand_source")"
    [ -n "$version" ] || { echo "bundle.sh: no 'const VERSION' in $brand_source" >&2; exit 1; }
fi
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "bundle.sh: version '$version' is not MAJOR.MINOR.PATCH" >&2
    exit 1
fi

case "$(uname -m)" in
    arm64) arch="arm64" ;;
    x86_64) arch="x64" ;;
    *) echo "bundle.sh: unsupported host architecture $(uname -m)" >&2; exit 1 ;;
esac

xml_escape() {
    local text="$1"
    text="${text//&/&amp;}"
    text="${text//</&lt;}"
    text="${text//>/&gt;}"
    text="${text//\"/&quot;}"
    printf '%s' "$text"
}

icon=""
for candidate in "$icon_dir/icon-$brand-$channel.icns" "$icon_dir/icon.icns"; do
    if [ -f "$candidate" ]; then
        icon="$candidate"
        break
    fi
done

write_info_plist() {
    local icon_entry=""
    if [ -n "$icon" ]; then
        icon_entry="    <key>CFBundleIconFile</key>
    <string>app.icns</string>
"
    fi
    cat <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>$(xml_escape "$display_name")</string>
    <key>CFBundleName</key>
    <string>$(xml_escape "$display_name")</string>
    <key>CFBundleExecutable</key>
    <string>$EXE</string>
    <key>CFBundleIdentifier</key>
    <string>$(xml_escape "$app_id")</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
${icon_entry}    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>$(xml_escape "$app_id")</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>$(xml_escape "$scheme")</string>
            </array>
        </dict>
    </array>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>LSMinimumSystemVersion</key>
    <string>$MIN_MACOS</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST
}

bundle="$dist/$display_name.app"
zip_name="$scheme-$version-macos-$arch.zip"

if [ "$dry_run" -eq 1 ]; then
    scratch="$(mktemp -d)"
    trap 'rm -rf "$scratch"' EXIT
    write_info_plist >"$scratch/Info.plist"
    plutil -lint "$scratch/Info.plist" >/dev/null
    echo "dry run: $brand/$channel $version ($arch)"
    echo "  bundle      $bundle"
    echo "  identifier  $app_id"
    echo "  URL scheme  $scheme://"
    echo "  executable  $release/$EXE -> Contents/MacOS/$EXE"
    echo "  icon        ${icon:-none}"
    echo "  archive     $dist/$zip_name"
    echo "  Info.plist (lint ok):"
    sed 's/^/    /' "$scratch/Info.plist"
    exit 0
fi

if [ "$skip_build" -eq 0 ]; then
    features=()
    for feature in "$brand_feature" "$channel_feature"; do
        if [ -n "$feature" ]; then
            features+=("sp-brand/$feature")
        fi
    done
    cargo_args=(build --release --locked -p harness-desktop)
    if [ "${#features[@]}" -gt 0 ]; then
        cargo_args+=(--features "$(IFS=,; echo "${features[*]}")")
    fi
    echo "cargo ${cargo_args[*]}"
    (cd "$workspace" && MACOSX_DEPLOYMENT_TARGET="$MIN_MACOS" cargo "${cargo_args[@]}")
fi

built="$release/$EXE"
[ -f "$built" ] || { echo "bundle.sh: missing $built; build first or drop --skip-build" >&2; exit 1; }

if [ -z "$icon" ]; then
    echo "warning: no icon in $icon_dir; the bundle has no CFBundleIconFile and shows the generic app icon" >&2
fi

mkdir -p "$dist"
rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp "$built" "$bundle/Contents/MacOS/$EXE"
chmod 755 "$bundle/Contents/MacOS/$EXE"
write_info_plist >"$bundle/Contents/Info.plist"
printf 'APPL????' >"$bundle/Contents/PkgInfo"
if [ -n "$icon" ]; then
    cp "$icon" "$bundle/Contents/Resources/app.icns"
fi
plutil -lint "$bundle/Contents/Info.plist" >/dev/null

codesign --force --sign - --timestamp=none "$bundle"
codesign --verify --strict --verbose=2 "$bundle"

rm -f "$dist/$zip_name"
ditto -c -k --sequesterRsrc --keepParent "$bundle" "$dist/$zip_name"
(cd "$dist" && shasum -a 256 "$zip_name" >SHA256SUMS)
echo "$(basename "$bundle")  $zip_name  $(cut -d' ' -f1 "$dist/SHA256SUMS")"
