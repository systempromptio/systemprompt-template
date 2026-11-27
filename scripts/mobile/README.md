# scripts/mobile

Scripts for the phone apps (`extensions/mobile`) and the `mobile` flavour. A
script lives here when a recipe in `mobile.just` and the `Mobile` workflow
both need it, so the two cannot drift; anything one recipe alone needs stays
inline in `mobile.just`.

| Script | Does |
|---|---|
| `toolchain.sh` | prints the pins in `extensions/mobile/shared/toolchain.env` and what this machine has (JDK, Android SDK packages, Xcode); fails when the Gradle wrapper, the version catalog, the Xcode project or the workflow disagree with the pins |
| `android-sdk.sh` | installs the pinned Android platform and build tools with `sdkmanager`, and the NDK with `--ndk` |

The build scripts themselves are part of the extension a fork takes with it:
`extensions/mobile/shared/build-android.sh` and `build-ios.sh`.

Every script here runs on macOS and Linux: no GNU-only flags.

The flavour-wide scripts stay in `scripts/`: `flavour.py` reads
`flavours/mobile/flavour.yaml`, and `flavour-build.sh`, `flavour-smoke.sh`,
`flavour-metrics.sh` and `flavour-setup.sh` exit early for any flavour that
is not an image.
