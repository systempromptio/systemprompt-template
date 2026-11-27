#!/usr/bin/env bash
# Screenshots of the Android app's first screen, one PNG per state.
#   android-screenshots.sh OUT                  the stub tour, into the directory OUT
#   android-screenshots.sh OUT --instance URL   one screenshot of the app after it discovers the instance at URL
# The stub tour drives the installed debug build through the states the screen
# has today: 01-launch.png (the title and the library's versions),
# 02-found.png (the stub instance discovered), 03-invalid-address.png and
# 04-unreachable.png. --instance writes 05-instance.png and prints the name
# the app showed; a loopback URL is forwarded over adb first, so a server on
# this machine is reached from the device at the same address.
# The app is driven the way a person drives it: the field and the button are
# found by their test tags in a uiautomator dump (the screen publishes them as
# resource ids), tapped, and typed into with `adb shell input`. What was typed
# is read back from the field and typed again when the device dropped a
# character. A capture that is one flat colour fails the run: an emulator
# without hardware acceleration draws nothing.
# Needs a connected device or emulator (just mobile-android-emulator), the
# debug APK (just mobile-android) and python3. The stub instance is started
# unless one is listening on MOBILE_STUB_PORT.
# Run by `just mobile-android-screenshots`.
set -euo pipefail
source "$(dirname "$0")/../../extensions/mobile/shared/env.sh"
[ -n "${ANDROID_HOME:-}" ] || mobile_fail "no Android SDK found: set ANDROID_HOME"
adb="$ANDROID_HOME/platform-tools/adb"
apk="$MOBILE_ROOT/android/app/build/outputs/apk/debug/app-debug.apk"
stub="$MOBILE_ROOT/shared/stub-instance.py"
usage() {
    echo "usage: android-screenshots.sh OUT [--instance URL]" >&2
    exit 2
}
out="${1:-}"
[ -n "$out" ] || usage
instance=""
case "${2:-}" in
    "") [ "$#" -eq 1 ] || usage ;;
    --instance)
        [ "$#" -eq 3 ] || usage
        instance="$3"
        ;;
    *) usage ;;
esac
[ "$("$adb" get-state 2> /dev/null || true)" = device ] \
    || mobile_fail "no device or emulator is connected, or more than one is (set ANDROID_SERIAL): just mobile-android-emulator starts one"
[ -f "$apk" ] || mobile_fail "no debug APK at $apk: just mobile-android builds it"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="$(mktemp -d)"
stub_pid=""
cleanup() {
    [ -z "$stub_pid" ] || kill "$stub_pid" 2> /dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT

tag_prefix=sign_in.instance
wait_seconds=60

# node FIELD TAG: the centre ("x y") or the text of the node whose resource id
# is TAG in the current window; exit 1 when the window has no such node.
node() {
    "$adb" exec-out uiautomator dump /dev/tty > "$work/window.xml" 2> /dev/null || true
    python3 - "$work/window.xml" "$1" "$2" << 'PY'
import re
import sys
import xml.etree.ElementTree as ET

path, field, tag = sys.argv[1:]
raw = open(path, encoding="utf-8", errors="replace").read()
start, end = raw.find("<hierarchy"), raw.rfind("</hierarchy>")
if start < 0 or end < 0:
    sys.exit(1)
root = ET.fromstring(raw[start:end + len("</hierarchy>")])
for found in root.iter("node"):
    if found.get("resource-id") == tag:
        if field == "text":
            print(found.get("text", ""))
        else:
            left, top, right, bottom = map(int, re.findall(r"-?\d+", found.get("bounds", "")))
            print((left + right) // 2, (top + bottom) // 2)
        sys.exit(0)
sys.exit(1)
PY
}

# wait_for TAG...: until one of the tags is on the screen; prints the one found.
wait_for() {
    local waited=0 tag
    while [ "$waited" -lt "$wait_seconds" ]; do
        for tag in "$@"; do
            if node text "$tag" > /dev/null; then
                echo "$tag"
                return 0
            fi
        done
        sleep 1
        waited=$((waited + 1))
    done
    mobile_fail "none of these appeared on the screen in $wait_seconds seconds: $*"
}

tap() {
    local centre
    centre="$(node centre "$1")" || mobile_fail "the screen has no $1 to tap"
    # shellcheck disable=SC2086
    "$adb" shell input tap $centre
}

hide_keyboard() {
    if "$adb" shell dumpsys input_method | grep -q 'mInputShown=true'; then
        "$adb" shell input keyevent KEYCODE_BACK
        sleep 1
    fi
}

launch() {
    "$adb" shell am force-stop "$MOBILE_APP_ID"
    "$adb" shell am start -W -n "$MOBILE_APP_ID/.MainActivity" > /dev/null
    wait_for root.title > /dev/null
}

# type_address TEXT: typed into the address field and read back; the field is
# emptied and the text typed again when what arrived is not what was sent.
type_address() {
    local text="$1" attempt=0 held="" erase
    while [ "$attempt" -lt 4 ]; do
        tap "$tag_prefix.address"
        sleep 1
        "$adb" shell input text "$text"
        sleep 1
        held="$(node text "$tag_prefix.address" || true)"
        [ "$held" != "$text" ] || return 0
        "$adb" shell input keyevent KEYCODE_MOVE_END
        erase=0
        while [ "$erase" -le "${#held}" ]; do
            "$adb" shell input keyevent KEYCODE_DEL
            erase=$((erase + 1))
        done
        attempt=$((attempt + 1))
    done
    mobile_fail "the device kept dropping characters: sent '$text', the field holds '$held'"
}

capture() {
    local file="$out/$1"
    hide_keyboard
    "$adb" exec-out screencap -p > "$file"
    python3 - "$file" << 'PY' || mobile_fail "the capture $1 is one flat colour: the device drew nothing (an emulator without hardware acceleration does this)"
import struct
import sys
import zlib

data = open(sys.argv[1], "rb").read()
if data[:8] != b"\x89PNG\r\n\x1a\n":
    sys.exit(1)
position, width, height, packed = 8, 0, 0, b""
while position < len(data):
    length, kind = struct.unpack(">I4s", data[position:position + 8])
    body = data[position + 8:position + 8 + length]
    if kind == b"IHDR":
        width, height = struct.unpack(">II", body[:8])
    elif kind == b"IDAT":
        packed += body
    position += 12 + length
pixels = zlib.decompress(packed)
stride = len(pixels) // height if height else 0
rows = {pixels[row * stride:(row + 1) * stride] for row in range(height)}
sys.exit(0 if width and len(rows) > 1 else 1)
PY
    echo "mobile: $file"
}

discover() {
    type_address "$1"
    hide_keyboard
    tap "$tag_prefix.discover"
    wait_for "$tag_prefix.display_name" "$tag_prefix.error"
}

forward() {
    "$adb" reverse "tcp:$1" "tcp:$1" > /dev/null || mobile_fail "adb could not forward port $1 to the device"
}

"$adb" install -r "$apk" > /dev/null

if [ -n "$instance" ]; then
    case "$instance" in
        http://127.0.0.1:* | http://localhost:*)
            port="${instance##*:}"
            forward "${port%%/*}"
            ;;
    esac
    launch
    shown="$(discover "$instance")"
    capture 05-instance.png
    [ "$shown" = "$tag_prefix.display_name" ] \
        || mobile_fail "the app did not find an instance at $instance: $(node text "$tag_prefix.error" || true)"
    echo "mobile: the app found '$(node text "$tag_prefix.display_name")' at $instance"
    exit 0
fi

if ! python3 "$stub" --probe --port "$MOBILE_STUB_PORT"; then
    python3 "$stub" --port "$MOBILE_STUB_PORT" > /dev/null &
    stub_pid=$!
    sleep 1
    python3 "$stub" --probe --port "$MOBILE_STUB_PORT" \
        || mobile_fail "the stub instance did not start on port $MOBILE_STUB_PORT"
fi
forward "$MOBILE_STUB_PORT"

launch
capture 01-launch.png

shown="$(discover "http://127.0.0.1:$MOBILE_STUB_PORT")"
[ "$shown" = "$tag_prefix.display_name" ] \
    || mobile_fail "the app did not find the stub instance: $(node text "$tag_prefix.error" || true)"
capture 02-found.png

launch
shown="$(discover demo.systemprompt.io)"
[ "$shown" = "$tag_prefix.error" ] || mobile_fail "an address with no scheme was accepted"
capture 03-invalid-address.png

# Why: port 9 (discard) has no listener on the device, so the connection is
# refused at once and the screen shows the unreachable message without a wait.
launch
shown="$(discover http://127.0.0.1:9)"
[ "$shown" = "$tag_prefix.error" ] || mobile_fail "an address nothing listens on was found"
capture 04-unreachable.png

echo "mobile: four states captured into $out"
