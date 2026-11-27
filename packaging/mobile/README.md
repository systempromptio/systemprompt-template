# packaging/mobile

What a release of the phone apps needs beyond the source in
`extensions/mobile`: signing configuration and export options. **Templates
only.** Every file here is an example with placeholders; no key, keystore,
certificate, provisioning profile or password is tracked, in this directory
or anywhere else, and `.gitignore` here refuses the real ones.

| File | Is |
|---|---|
| `android/keystore.properties.example` | the four values the Android release signing configuration reads |
| `ios/ExportOptions.example.plist` | the export options `xcodebuild -exportArchive` reads for a store-connect upload |

## State

Nothing reads these yet. The scaffold builds unsigned: the Android debug APK
carries only the debug key Gradle generates on each machine, and the iOS
simulator build is not signed at all. Release signing, the archive and export
steps, and store metadata arrive with the build and distribution stage.

## Android

Each publisher signs its own package name with its own key.

1. Create a keystore outside the repository
   (`keytool -genkeypair -v -keystore <path> -alias <alias> -keyalg RSA -keysize 4096 -validity 9125`).
2. Copy `android/keystore.properties.example` to
   `extensions/mobile/android/keystore.properties` (untracked) and fill it
   in. On a runner the same four values come from the workflow's secrets.
3. Register the package name and the key's fingerprint for developer
   verification before distributing outside a managed store.

## iOS

Each publisher signs its own bundle identifier with its own distribution
certificate and provisioning profile, and uploads with its own store-connect
key.

1. Copy `ios/ExportOptions.example.plist` to `ExportOptions.plist`
   (untracked) and set the team identifier and the profile name.
2. Archive and export on macOS:
   `xcodebuild archive`, then `xcodebuild -exportArchive -exportOptionsPlist ExportOptions.plist`.

The template signs the reference app (`io.systemprompt.mobile`) with the
maintainers' identities; a fork signs its own brand with its own.
