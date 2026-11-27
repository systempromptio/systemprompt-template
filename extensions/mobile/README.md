# extensions/mobile

The phone apps: an iOS app in SwiftUI and an Android app in Jetpack Compose,
written twice and kept alike. This is the `mobile` flavour
(`flavours/mobile/`). The apps are clients of an instance: they run nothing
locally, and everything that is behaviour (sign-in, sync, typed calls, event
streams, a state machine per feature) will come from one shared Rust library
that both apps call through generated bindings.

Xcode builds `ios/` and Gradle builds `android/`. `core/` is the Rust
binding library both apps load: a cargo workspace of its own, excluded from
the root workspace, so the server build never compiles a phone target and the
phone build never compiles the server. The server image sees none of this
(`.dockerignore`).

## State

The Android app calls real Rust through generated Kotlin. Its one screen,
"enter an instance", takes an address, asks the shared library to read that
instance's discovery document and shows what came back: the display name, the
sign-in hint and where the instance offers each platform's app, or one message
per error the library can return. Beneath it are the binding library's version
and the core version the library reports. There is no sign-in yet.

The iOS app is still the blank, themed screen showing the app name. It does
not link the library: its Rust library step builds nothing and says so.

The Android app builds and passes its checks on Linux. **The iOS project was
written by hand on a machine with no Xcode: it has never been opened or
built.** Its first build is the `Mobile` workflow's macOS job.

## Layout

```
extensions/mobile/
  ios/                         an Xcode project (Mobile.xcodeproj, scheme Mobile)
    App/                       entry point, the root screen, strings (Localizable.xcstrings), assets
    Design/                    the theme and palette; components over generated brand assets
    Features/                  one directory per screen group (none yet)
    Tests/  UITests/           unit tests; launch and UI tests
  android/                     a Gradle project (Kotlin DSL, wrapper committed)
    app/                       entry point, the application class that starts the library; src/test, src/androidTest
    design/                    the theme and palette; components over generated brand resources
    feature/sign-in/           the "enter an instance" screen, its view model and strings
    gradle/libs.versions.toml  the version catalog, which is the dependency allowlist
  core/                        the binding library: a cargo workspace of one crate, with its own Cargo.lock
    src/lib.rs                 core's surface re-exported, plus this crate's own namespace
    src/android.rs             the one JNI entry point (Android only)
    src/bin/uniffi-bindgen.rs  the generator, from the same UniFFI release the library links
  shared/
    toolchain.env              every pinned version
    env.sh                     sourced by the scripts: the pins, ANDROID_HOME, the Rust library step
    build-android.sh           the Rust library step, then gradlew assembleDebug, check or the instrumented tests
    build-ios.sh               the Rust library step (a stub), then xcodebuild for the simulator, or the linters
    stub-instance.py           a stub instance that serves the discovery document, for development and the tests
    screens.yaml               the screens and states both apps draw
```

## The binding library

The apps do not import Rust source. `core/` links core's `systemprompt-mobile`
crate, which annotates the shared surface (`Instances`, `diagnostics()`, the
records they return and the one error type), adds this crate's own annotated
items (`library_version()` today) and builds one library. The generator reads
that library and writes one Kotlin package for each annotated crate:
`uniffi.systemprompt_mobile` and `uniffi.systemprompt_mobile_app`. A fork
exports its own Rust to the apps by annotating it in `core/`, without
changing core.

`build-android.sh` runs the step before Gradle:

1. `cargo ndk` builds `libsystemprompt_mobile_app.so` for arm64-v8a,
   armeabi-v7a and x86_64 into `android/app/src/main/jniLibs/`.
2. The crate's `uniffi-bindgen` binary (feature `cli`, built for the host)
   generates Kotlin from one of those libraries into
   `android/app/build/generated/uniffi/kotlin/`, which `:app` compiles.

Both are build output and neither is tracked. The Kotlin calls the library
through JNA, and its `suspend` functions use kotlinx-coroutines.

The library's HTTP client verifies certificates with the system's verifier,
which on Android is reached through the JVM. `MobileApplication` therefore
loads the library and calls `NativeLibrary.initialisePlatform` with the
application context once, before anything else; the Kotlin half of that
verifier is the `org.rustls:rustls-platform-verifier` artefact, whose version
Gradle reads from `core/Cargo.lock` so the two halves cannot differ.

The core the crate links is a git dependency on the rev in `bridge/CORE_REF`.
`scripts/core-pin.sh` moves it with the root's and `tests/`' entries and
re-resolves `core/Cargo.lock`; `scripts/core-patch.sh on <checkout>` switches
all three to a local core checkout. The lint table in `core/Cargo.toml` is the
root's, copied; keep them identical.

The two apps mirror each other's structure and names, so a change is made in
the same place twice: `Design/BrandPalette.swift` and
`design/.../BrandPalette.kt`, the string `app_name` in both catalogues, the
test id `root.title` in both launch tests. Until the iOS app links the
library, its root screen is still `App/RootScreen.swift`, and the Android
screen under `feature/sign-in` has no iOS counterpart.

The Xcode project uses file-system-synchronised groups: a source file added to
`App/`, `Design/`, `Features/`, `Tests/` or `UITests/` is part of its
target with no edit to `project.pbxproj`. On Android, `design/` and
`feature/sign-in/` are compiled by `:app` as further source directories until
each becomes a module of its own.

## Building

```bash
just mobile-toolchain          # the pins, what this machine has, and whether the pins agree
just mobile-android-sdk --ndk  # install the pinned platform, build tools and NDK with sdkmanager
just mobile-rust-targets       # add the pinned Android Rust targets to the pinned toolchain
just mobile-android-library    # the shared library for every Android ABI, and its Kotlin
just mobile-android-check      # rustfmt and clippy over core/; Android lint, ktlint, detekt, the JVM unit tests
just mobile-android            # the debug APK
just mobile-android-install    # install it on the connected device or emulator and launch it
just mobile-android-emulator   # create and start a headless emulator (stop: just mobile-android-emulator stop)
just mobile-android-test       # the instrumented tests on it, against the stub instance
just mobile-android-screenshots OUT   # one PNG per state of the first screen, into the directory OUT
just mobile-stub-instance      # the stub instance alone, for trying the app by hand
just mobile-ios-check          # SwiftLint and swift-format (macOS only)
just mobile-ios                # the simulator build, unsigned (macOS only)
```

Android needs JDK 17 and an Android SDK with the pinned NDK: `ANDROID_HOME`,
or the SDK in its default place (`~/Android/Sdk` on Linux,
`~/Library/Android/sdk` on macOS). The library step needs the Rust toolchain
in `rust-toolchain.toml` with the three Android targets, and cargo-ndk at
the pinned version (`cargo install cargo-ndk --version 4.1.2 --locked`).
`MOBILE_RUST_PROFILE=release` builds the library with the release profile.
iOS needs macOS with Xcode 16 or later; on any other system `just mobile-ios`
refuses and says so. `shared/toolchain.env` holds every version;
`scripts/mobile/toolchain.sh` fails when the Gradle wrapper, the version
catalog, the binding crate's manifest, the Xcode project or the workflow
disagree with it, or when `core/Cargo.lock` does not resolve core at
`bridge/CORE_REF`.

## Tests

The JVM unit tests (`just mobile-android-check`) cover the view model: it is
written against a small interface, `InstanceDirectory`, so a test stands in
for the shared library, which only loads on a device.

The instrumented tests (`just mobile-android-test`) need a running emulator
or device. They launch the app, assert it shows the core version the library
reports, type the stub instance's address and assert the stub's display name
appears. `shared/stub-instance.py` serves the discovery document on this
machine's loopback, port 8765 (`MOBILE_STUB_PORT` moves it), which an
emulator reaches at `http://10.0.2.2:8765`; the recipe starts it when none is
listening. The library admits plain HTTP for loopback and private addresses
only, and the app permits it in the debug build only.

`just mobile-android-screenshots OUT` drives the installed debug build on the
same emulator or device and writes one PNG per state of the first screen:
launch, the stub instance found, an invalid address and an unreachable one.
`--instance URL` writes one more against a real instance; a loopback URL is
forwarded over adb first. It finds the field and the button by their test
tags, which the screen publishes as resource ids, reads back what it typed,
and fails on a capture that is one flat colour, which is what an emulator
without hardware acceleration produces.

`shared/stub-instance.py --releases APK` is a second stand-in, for the
release API an instance reads its Android download from: it serves one
release whose assets are that file and its `SHA256SUMS`. Point a local
instance's `gateway.bridge_releases.api_base` at it to try
`/v1/bridge/latest` and `/v1/bridge/download/android` without a GitHub
release.

The iOS tests (`xcodebuild test`) need a simulator; no recipe starts one yet.

The `Mobile` workflow (`.github/workflows/mobile.yml`) builds the shared
library, runs the Android checks and builds the debug APK on Linux, and runs
the iOS checks and simulator build on macOS. Both builds are unsigned and
store nothing. No job starts an emulator.

## Standards

| Concern | iOS | Android |
|---|---|---|
| Formatting | swift-format (`ios/.swift-format`), `lint --strict` | ktlint (`android/.editorconfig`) |
| Linting | SwiftLint (`ios/.swiftlint.yml`), `--strict` | detekt (`android/config/detekt/detekt.yml`) and Android lint |
| Warnings | errors, in the compiler and the linters | errors, in the compiler and the linters |
| No behaviour in the app | a SwiftLint rule refuses direct networking | a detekt rule refuses networking imports |
| Strings | the String Catalog, never a literal | string resources, never a literal |
| Dependencies | none; a package is a reviewed change | the version catalog; an entry is a reviewed change |
| Rust (`core/`) | rustfmt, clippy with the root's lint table, warnings are errors | the same crate |

No credential, key, keystore or provisioning profile is tracked. Signing
templates are in `packaging/mobile/`.

## What a fork owns

A fork takes this directory with its other extensions and ships its own
branded apps.

| A fork changes freely | A fork cannot change |
|---|---|
| Brand, icons, product name, default instance, identifiers | How credentials are sealed; that the native side holds no secret |
| Themes | Manifest trust and the key-trust prompt's existence |
| Any screen, navigation and layout | The approval rule: only the owner decides, a timeout denies |
| Which optional features are shown | Features the instance disables or requires |
| Its own Rust members for logic | The push payload carrying no content |

The reference app's identifier is `io.systemprompt.mobile` (the Android
`applicationId` and `namespace`, the iOS bundle identifier) and its display
name is the string `app_name`. A fork changes the identifier in
`shared/toolchain.env`, `android/app/build.gradle.kts` and the Xcode
project's build settings, and the name in both string catalogues and the
project's `INFOPLIST_KEY_CFBundleDisplayName`. A later build step generates
all of these, and the palettes and icons, from one brand declaration.

## Licence

MIT, like the rest of the template. The apps link core's crates through the
binding library, as the web and desktop extensions link them.
