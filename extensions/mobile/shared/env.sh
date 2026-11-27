# shellcheck shell=bash
# Sourced by the mobile scripts: loads toolchain.env and finds the Android SDK.
# Sets MOBILE_ROOT (extensions/mobile), every MOBILE_* pin, and ANDROID_HOME
# when it is unset and the SDK sits in its default place for this system, and
# defines the Rust library step of each platform's build. MOBILE_STUB_PORT is
# where the stub instance (stub-instance.py) listens; set it to move it.
MOBILE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
set -a
# shellcheck source=toolchain.env
source "$MOBILE_ROOT/shared/toolchain.env"
set +a

if [ -z "${ANDROID_HOME:-}" ]; then
    for candidate in "${ANDROID_SDK_ROOT:-}" "$HOME/Android/Sdk" "$HOME/Library/Android/sdk"; do
        if [ -n "$candidate" ] && [ -d "$candidate" ]; then
            export ANDROID_HOME="$candidate"
            break
        fi
    done
fi

# The binding crate, the library it builds, and where the Android build puts
# what it makes of it. Both directories are build output and never tracked.
MOBILE_CORE="$MOBILE_ROOT/core"
MOBILE_LIBRARY=systemprompt_mobile_app
MOBILE_ANDROID_JNILIBS="$MOBILE_ROOT/android/app/src/main/jniLibs"
MOBILE_ANDROID_BINDINGS="$MOBILE_ROOT/android/app/build/generated/uniffi/kotlin"
MOBILE_STUB_PORT="${MOBILE_STUB_PORT:-8765}"

mobile_fail() {
    echo "mobile: $1" >&2
    exit 1
}

# The Rust library step of the Android build: the binding crate compiled for
# every ABI in MOBILE_RUST_TARGETS_ANDROID into the app's jniLibs, then the
# Kotlin generated from one of those libraries by the crate's own generator.
# MOBILE_RUST_PROFILE picks the cargo profile: dev (the default) or release.
mobile_rust_library_android() {
    local profile="${MOBILE_RUST_PROFILE:-dev}" ndk library target
    local -a targets=() flags=(--locked)
    [ -n "${ANDROID_HOME:-}" ] || mobile_fail "no Android SDK found: set ANDROID_HOME"
    ndk="$ANDROID_HOME/ndk/$MOBILE_ANDROID_NDK"
    [ -d "$ndk" ] || mobile_fail "the NDK $MOBILE_ANDROID_NDK is not installed (just mobile-android-sdk --ndk)"
    [ "$(cargo ndk --version 2> /dev/null)" = "cargo-ndk $MOBILE_CARGO_NDK" ] \
        || mobile_fail "cargo-ndk $MOBILE_CARGO_NDK is not installed (cargo install cargo-ndk --version $MOBILE_CARGO_NDK --locked)"
    case "$profile" in
        dev) ;;
        release) flags+=(--release) ;;
        *) mobile_fail "MOBILE_RUST_PROFILE is '$profile'; it is dev or release" ;;
    esac
    for target in $(printf '%s' "$MOBILE_RUST_TARGETS_ANDROID" | tr ',' ' '); do
        targets+=(--target "$target")
    done
    rm -rf "$MOBILE_ANDROID_JNILIBS" "$MOBILE_ANDROID_BINDINGS"
    (
        cd "$MOBILE_CORE"
        export CARGO_TARGET_DIR="$MOBILE_CORE/target" ANDROID_NDK_HOME="$ndk"
        cargo ndk "${targets[@]}" --platform "$MOBILE_ANDROID_MIN_SDK" --output-dir "$MOBILE_ANDROID_JNILIBS" \
            build "${flags[@]}"
        library="$(find "$MOBILE_ANDROID_JNILIBS" -name "lib$MOBILE_LIBRARY.so" | sort | head -1)"
        [ -n "$library" ] || mobile_fail "the build left no lib$MOBILE_LIBRARY.so under $MOBILE_ANDROID_JNILIBS"
        cargo run --locked --quiet --features cli --bin uniffi-bindgen -- \
            generate --language kotlin --no-format --config "$MOBILE_CORE/uniffi-android.toml" \
            --out-dir "$MOBILE_ANDROID_BINDINGS" "$library"
    )
    find "$MOBILE_ANDROID_JNILIBS" -name "lib$MOBILE_LIBRARY.so" | sort | while read -r library; do
        echo "mobile: $library ($(wc -c < "$library" | tr -d ' ') bytes, $profile)"
    done
    echo "mobile: Kotlin bindings generated into $MOBILE_ANDROID_BINDINGS"
}

# Formatting and clippy over the binding crate, every warning an error.
# Clippy runs for one Android target, so the JNI module is linted too.
mobile_rust_lint() {
    local ndk target
    [ -n "${ANDROID_HOME:-}" ] || mobile_fail "no Android SDK found: set ANDROID_HOME"
    ndk="$ANDROID_HOME/ndk/$MOBILE_ANDROID_NDK"
    [ -d "$ndk" ] || mobile_fail "the NDK $MOBILE_ANDROID_NDK is not installed (just mobile-android-sdk --ndk)"
    target="${MOBILE_RUST_TARGETS_ANDROID##*,}"
    (
        cd "$MOBILE_CORE"
        export CARGO_TARGET_DIR="$MOBILE_CORE/target" ANDROID_NDK_HOME="$ndk"
        cargo fmt --all -- --check
        cargo ndk --target "$target" --platform "$MOBILE_ANDROID_MIN_SDK" clippy --locked -- -D warnings
    )
}

# The Rust library step of the iOS build. Not wired yet: the binding crate
# (core/) builds a static library, but nothing compiles it for the iOS targets,
# assembles the XCFramework or generates the Swift, so the app builds alone.
mobile_rust_library_ios() {
    echo "mobile: the iOS app does not link the binding library yet ($MOBILE_RUST_TARGETS_IOS): the static libraries, the XCFramework and the generated Swift are not built, so the app is built alone"
}
