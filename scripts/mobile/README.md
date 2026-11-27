# scripts/mobile

Scripts for the phone apps (`extensions/mobile`) and the `mobile` flavour. A
script lives here when a recipe in `mobile.just` and the `Mobile` workflow
both need it, so the two cannot drift; anything one recipe alone needs stays
inline in `mobile.just`.

| Script | Does |
|---|---|
| `toolchain.sh` | prints the pins in `extensions/mobile/shared/toolchain.env` and what this machine has (JDK, Android SDK packages, Rust targets, cargo-ndk, Xcode); fails when the Gradle wrapper, the version catalog, the binding crate's manifest, the Xcode project or the workflow disagree with the pins, or when the binding crate's lockfile does not resolve core at `bridge/CORE_REF` |
| `android-sdk.sh` | installs the pinned Android platform and build tools with `sdkmanager`, and the NDK with `--ndk` |
| `rust-targets.sh` | adds the pinned Rust targets for `android` or `ios` to the toolchain `rust-toolchain.toml` names |
| `android-emulator.sh` | creates the AVD from the pinned system image when it is missing and starts it headless (`start`, `run`, `wait`, `stop`); local only, no workflow runs it |
| `android-screenshots.sh` | installs the debug APK on the connected device or emulator, drives the first screen by its test tags and writes one PNG per state (launch, found, invalid address, unreachable) into a directory; `--instance URL` captures one against a real instance; fails on a capture that is one flat colour; local only |
| `ios-tools.sh` | installs the pinned SwiftLint binary, checked against its pinned sha256, into `extensions/mobile/ios/build/tools` (macOS only) |

The build scripts themselves are part of the extension a fork takes with it:
`extensions/mobile/shared/build-android.sh` and `build-ios.sh`, with the
Rust library step in `env.sh` and the stub instance the instrumented test
reads (`stub-instance.py`).

Every script here runs on macOS and Linux: no GNU-only flags.

The flavour-wide scripts stay in `scripts/`: `flavour.py` reads
`flavours/mobile/flavour.yaml`, and `flavour-build.sh`, `flavour-smoke.sh`,
`flavour-metrics.sh` and `flavour-setup.sh` exit early for any flavour that
is not an image.
