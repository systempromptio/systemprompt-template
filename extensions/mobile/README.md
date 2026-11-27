# extensions/mobile

The phone apps: an iOS app in SwiftUI and an Android app in Jetpack Compose,
written twice and kept alike. This is the `mobile` flavour
(`flavours/mobile/`). The apps are clients of an instance: they run nothing
locally, and everything that is behaviour (sign-in, sync, typed calls, event
streams, a state machine per feature) will come from one shared Rust library
that both apps call through generated bindings.

Nothing here is part of a cargo workspace. Xcode builds `ios/`, Gradle builds
`android/`, and the server image never sees either (`.dockerignore`).

## State

A scaffold. Each app is one blank, themed screen showing the app name, with
its formatter, linter, one unit test and one launch test. There is no
networking, no sign-in and no feature. The Rust library step in both build
scripts is a stub: the binding library arrives with core's
`systemprompt-mobile` crate, and until then the scripts say so and build the
app alone.

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
    app/                       entry point, the root screen, strings and resources; src/test, src/androidTest
    design/                    the theme and palette; components over generated brand resources
    feature/                   one module per screen group (none yet)
    gradle/libs.versions.toml  the version catalog, which is the dependency allowlist
  shared/
    toolchain.env              every pinned version
    env.sh                     sourced by the scripts: the pins, ANDROID_HOME, the Rust library stub
    build-android.sh           the Rust library step (stub), then gradlew assembleDebug or check
    build-ios.sh               the Rust library step (stub), then xcodebuild for the simulator, or the linters
    screens.yaml               the screens and states both apps draw
```

The two apps mirror each other's structure and names, so a change is made in
the same place twice: `App/RootScreen.swift` and
`app/.../RootScreen.kt`, `Design/BrandPalette.swift` and
`design/.../BrandPalette.kt`, the string `app_name` in both catalogues, the
test id `root.title` in both launch tests.

The Xcode project uses file-system-synchronised groups: a source file added to
`App/`, `Design/`, `Features/`, `Tests/` or `UITests/` is part of its
target with no edit to `project.pbxproj`. On Android, `design/` is compiled
by `:app` as a second source directory until it becomes a module of its own.

## Building

```bash
just mobile-toolchain          # the pins, what this machine has, and whether the pins agree
just mobile-android-sdk        # install the pinned platform and build tools with sdkmanager
just mobile-android-check      # Android lint, ktlint, detekt, the JVM unit tests
just mobile-android            # the debug APK
just mobile-android-install    # install it on the connected device or emulator and launch it
just mobile-ios-check          # SwiftLint and swift-format (macOS only)
just mobile-ios                # the simulator build, unsigned (macOS only)
```

Android needs JDK 17 and an Android SDK: `ANDROID_HOME`, or the SDK in its
default place (`~/Android/Sdk` on Linux, `~/Library/Android/sdk` on macOS).
iOS needs macOS with Xcode 16 or later; on any other system `just mobile-ios`
refuses and says so. `shared/toolchain.env` holds every version;
`scripts/mobile/toolchain.sh` fails when the Gradle wrapper, the version
catalog, the Xcode project or the workflow disagree with it.

The instrumented launch test (`./gradlew connectedDebugAndroidTest` in
`android/`) and the iOS tests (`xcodebuild test`) need a running emulator or
simulator; no recipe or workflow job starts one yet.

The `Mobile` workflow (`.github/workflows/mobile.yml`) runs the Android
checks and debug build on Linux and the iOS checks and simulator build on
macOS. Both are unsigned and store nothing.

## Standards

| Concern | iOS | Android |
|---|---|---|
| Formatting | swift-format (`ios/.swift-format`), `lint --strict` | ktlint (`android/.editorconfig`) |
| Linting | SwiftLint (`ios/.swiftlint.yml`), `--strict` | detekt (`android/config/detekt/detekt.yml`) and Android lint |
| Warnings | errors, in the compiler and the linters | errors, in the compiler and the linters |
| No behaviour in the app | a SwiftLint rule refuses direct networking | a detekt rule refuses networking imports |
| Strings | the String Catalog, never a literal | string resources, never a literal |
| Dependencies | none; a package is a reviewed change | the version catalog; an entry is a reviewed change |

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

MIT, like the rest of the template. The apps will link core's crates through
the generated library, as the web and desktop extensions link them.
