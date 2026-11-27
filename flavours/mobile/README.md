# mobile

The phone apps: an iOS app and an Android app, clients of an instance. It is
the one flavour that produces neither a server image nor desktop installers,
which `artefact: mobile` in `flavour.yaml` says.

It is built from `extensions/mobile`, which holds an Xcode project
(`ios/`), a Gradle project (`android/`), the Rust binding library both load
(`core/`, a cargo workspace of its own) and the scripts they share
(`shared/`). None of it is part of the root cargo workspace. It names no root cargo
feature, ships no `services/` subtree and has no profile overlay:
`flavour.yaml` holds `name`, `description`, `artefact` and `workspace`,
and `scripts/flavour.py check` refuses the image keys here and requires the
two projects to exist.

## What works today

The Android app calls the shared Rust library through generated Kotlin: it
shows the library's diagnostics and discovers an instance from an address
typed into its one screen. It builds, lints and passes its unit tests on
Linux. The iOS app is a blank, themed screen showing the app name and does
not link the library yet. The iOS project has never
been opened or built on a Mac; the `Mobile` workflow's macOS job is its first
build.

```bash
just flavours                 # lists mobile beside the other flavours
just mobile-toolchain         # the pins, and whether every file agrees with them
just mobile-android-check     # Android lint, ktlint, detekt, unit tests
just mobile-android           # the debug APK
just mobile-android-install   # install and launch on a connected device or emulator
just mobile-ios               # the simulator build (macOS only)
```

The flavour recipes accept the name and say so rather than failing:
`just docker-build mobile`, `just smoke mobile`, `just metrics mobile` and
`just setup mobile` each print that the phone apps are built by the
`mobile-*` recipes and exit 0. `just metrics all` and the `Flavours`
workflow measure the image flavours only.

The `Mobile` workflow (`.github/workflows/mobile.yml`) builds the Android
app on Linux and the iOS app for the simulator on macOS, unsigned, and stores
no artefact.

## What it becomes

A build of this flavour compiles the shared Rust library for the six phone
targets, generates the Swift and Kotlin bindings, and produces a signed APK
the instance serves and a signed iOS build for private distribution. Signing
configuration templates are in `packaging/mobile/`; no key is tracked.
