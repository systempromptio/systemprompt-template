# Install Cowork on macOS

The Cowork Desktop app is the **client-side companion** to a running [systemprompt-gateway](../install/ghcr.md). It now ships as a native `.app` bundle (`Systemprompt Cowork.app`) wrapping the `systemprompt-bridge` binary — credential helper, signed-manifest sync agent, and local inference proxy in a single launchable app.

If you're looking to deploy the **server** (the gateway), see [../install/](../install/) instead.

For the full GUI tour, see [desktop-app.md](desktop-app.md).

> **v0.7.0 rename note.** The crate and binary were renamed `cowork → bridge` in the v0.7.0 release. The macOS `.app` is still branded "Systemprompt Cowork" and Claude Desktop's host integration label is unchanged. Inside the bundle the executable is `systemprompt-cowork` (parity with prior installs); on Linux/Windows the binary is `systemprompt-bridge`.

## Option 1 — Homebrew tap (recommended)

```bash
brew tap systempromptio/tap
brew install --cask cowork
open -a "Systemprompt Cowork"
```

The cask installs `Systemprompt Cowork.app` into `/Applications` and symlinks `systemprompt-bridge` onto your `PATH`.

## Option 2 — direct download from GitHub Releases

The branded `.app`/`.dmg` bundles are published on the template repo under the `cowork-v*` tag series (artifact names retained for backward compatibility); the raw `systemprompt-bridge` binaries ship on the `bridge-v*` track. Pick the matching architecture:

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -sSL -o SystempromptCowork.dmg \
  https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.7.0/SystempromptCowork-aarch64-apple-darwin.dmg

# Intel Mac
curl -sSL -o SystempromptCowork.dmg \
  https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.7.0/SystempromptCowork-x86_64-apple-darwin.dmg

hdiutil attach SystempromptCowork.dmg
cp -R "/Volumes/Systemprompt Cowork/Systemprompt Cowork.app" /Applications/
hdiutil detach "/Volumes/Systemprompt Cowork"
xattr -dr com.apple.quarantine "/Applications/Systemprompt Cowork.app"
open -a "Systemprompt Cowork"
```

If you only need the headless binary (Claude Desktop credential-helper slot, CI):

```bash
curl -sSL -o systemprompt-bridge \
  https://github.com/systempromptio/systemprompt-template/releases/download/bridge-v0.9.0/systemprompt-bridge-aarch64-apple-darwin
chmod +x systemprompt-bridge
sudo install -m 0755 systemprompt-bridge /usr/local/bin/systemprompt-bridge
```

Verify the SHA256:

```bash
curl -sSL -O https://github.com/systempromptio/systemprompt-template/releases/download/bridge-v0.9.0/SHA256SUMS.bridge
shasum -a 256 -c SHA256SUMS.bridge --ignore-missing
```

Verify the cosign signature:

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-core/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature SHA256SUMS.bridge.sig \
  --certificate SHA256SUMS.bridge.pem \
  SHA256SUMS.bridge
```

## First-run

Launch `Systemprompt Cowork.app` and the **Setup wizard** opens automatically. Two steps:

1. Paste the gateway URL and a PAT (or pick session / mTLS).
2. Enable each host integration you want managed (Claude Desktop, Codex CLI, …). Hosts default to **disabled** — nothing is silently probed.

Once setup completes, the proxy starts on `127.0.0.1:48217` and the activity drawer shows live request flow. The wizard can be re-run any time from **Settings → Re-run setup**.

CLI alternative (no GUI):

```bash
systemprompt-bridge login sp-live-... --gateway https://your-gateway.example.com
systemprompt-bridge install --apply --pubkey <base64>
systemprompt-bridge status
```

See [device-auth.md](device-auth.md) for the auth-mode options.

## Uninstall

```bash
brew uninstall --cask cowork                     # Homebrew
rm -rf "/Applications/Systemprompt Cowork.app"   # manual install
rm /usr/local/bin/systemprompt-bridge            # headless binary
```

To also wipe credentials and cache:

```bash
systemprompt-bridge uninstall --purge
```

This removes `~/.config/systemprompt/systemprompt-bridge.toml`, `~/Library/Caches/systemprompt-bridge/cache.json`, and `~/.config/systemprompt/agents.json`.

## Links

- [Desktop app overview](desktop-app.md)
- [Device auth modes](device-auth.md)
- [systemprompt.io](https://systemprompt.io/?utm_source=cowork-macos&utm_medium=install_doc)
- [Documentation](https://systemprompt.io/documentation/?utm_source=cowork-macos&utm_medium=install_doc)
- [systemprompt-template on GitHub](https://github.com/systempromptio/systemprompt-template)
- [LICENSE](../../LICENSE)
