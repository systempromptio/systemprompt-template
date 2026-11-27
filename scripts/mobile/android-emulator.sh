#!/usr/bin/env bash
# A headless Android emulator for the instrumented tests.
#   android-emulator.sh start    create the AVD if it is missing, start it in the background, wait for it to boot
#   android-emulator.sh run      the same, in the foreground until interrupted (no wait)
#   android-emulator.sh wait     wait for a started emulator to finish booting
#   android-emulator.sh stop     shut the emulator down
# The system image is the one extensions/mobile/shared/toolchain.env pins, for
# this machine's architecture; it is installed with sdkmanager when absent. The
# emulator needs hardware virtualisation: on Linux this user must be able to
# open /dev/kvm (membership of the kvm group), and the script says so when it
# cannot. Hosted Linux runners do not all offer it, so no workflow runs this.
# Run by `just mobile-android-emulator`.
set -euo pipefail
source "$(dirname "$0")/../../extensions/mobile/shared/env.sh"
[ -n "${ANDROID_HOME:-}" ] || mobile_fail "no Android SDK found: set ANDROID_HOME"
case "$(uname -m)" in
    arm64 | aarch64) abi=arm64-v8a ;;
    *) abi=x86_64 ;;
esac
image="system-images;android-$MOBILE_ANDROID_EMULATOR_API;$MOBILE_ANDROID_EMULATOR_TAG;$abi"
avd="$MOBILE_ANDROID_AVD"
adb="$ANDROID_HOME/platform-tools/adb"
emulator="$ANDROID_HOME/emulator/emulator"
tools="$ANDROID_HOME/cmdline-tools/latest/bin"
log="${TMPDIR:-/tmp}/mobile-android-emulator.log"
boot_seconds=300

ensure_avd() {
    [ -x "$emulator" ] || mobile_fail "the emulator is not installed (sdkmanager emulator)"
    if [ "$(uname -s)" = Linux ] && { [ ! -r /dev/kvm ] || [ ! -w /dev/kvm ]; }; then
        mobile_fail "this user cannot open /dev/kvm, which the emulator needs: add the user to the kvm group (sudo usermod -aG kvm \"$USER\") and log in again"
    fi
    if [ ! -d "$ANDROID_HOME/system-images/android-$MOBILE_ANDROID_EMULATOR_API/$MOBILE_ANDROID_EMULATOR_TAG/$abi" ]; then
        set +o pipefail
        yes | "$tools/sdkmanager" --sdk_root="$ANDROID_HOME" "$image" > /dev/null
        status="${PIPESTATUS[1]}"
        set -o pipefail
        [ "$status" -eq 0 ] || mobile_fail "sdkmanager failed installing $image"
    fi
    if ! "$emulator" -list-avds 2> /dev/null | grep -qx "$avd"; then
        echo no | "$tools/avdmanager" create avd --name "$avd" --package "$image" --force > /dev/null
        echo "mobile: created the AVD $avd from $image"
    fi
}

flags=(-avd "$avd" -no-window -no-audio -no-boot-anim -no-snapshot -gpu swiftshader_indirect)

wait_for_boot() {
    local waited=0
    until [ "$("$adb" get-state 2> /dev/null || true)" = device ] \
        && [ "$("$adb" shell getprop sys.boot_completed 2> /dev/null | tr -d '\r')" = 1 ]; do
        [ "$waited" -lt "$boot_seconds" ] || mobile_fail "the emulator did not boot in $boot_seconds seconds (log: $log)"
        sleep 5
        waited=$((waited + 5))
    done
    echo "mobile: the emulator $avd has booted"
}

case "${1:-start}" in
    start)
        ensure_avd
        "$emulator" "${flags[@]}" > "$log" 2>&1 &
        echo "mobile: starting the emulator $avd (log: $log)"
        wait_for_boot
        ;;
    run)
        ensure_avd
        exec "$emulator" "${flags[@]}"
        ;;
    wait) wait_for_boot ;;
    stop)
        "$adb" emu kill > /dev/null 2>&1 || true
        echo "mobile: the emulator was told to shut down"
        ;;
    *)
        echo "usage: android-emulator.sh [start|run|wait|stop]" >&2
        exit 2
        ;;
esac
