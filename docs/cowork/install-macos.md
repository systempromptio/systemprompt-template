# Install the systemprompt bridge on macOS

The bridge is the **client-side companion** to a running [systemprompt-gateway](../install/ghcr.md). It ships as a single `systemprompt-bridge` binary that is, at once, the credential helper, the signed-manifest sync agent, the local inference proxy, and a native desktop GUI (`systemprompt-bridge gui`).

If you're looking to deploy the **server** (the gateway), see [../install/](../install/) instead.

For the full GUI tour, see [desktop-app.md](desktop-app.md).

> **Naming note.** The crate and binary were renamed `cowork → bridge` in v0.7.0; the binary is `systemprompt-bridge` on macOS, Linux, and Windows. The Claude Desktop host-integration label is still "Cowork". The Homebrew formula and Scoop package are now named `bridge`; the legacy `cowork` package is retained for existing installs.

## Option 1 — Homebrew tap (recommended)

```bash
brew tap systempromptio/tap
brew install systempromptio/tap/bridge
systemprompt-bridge gui
```

The formula installs `systemprompt-bridge` onto your `PATH`. Launch the GUI with `systemprompt-bridge gui`.

## Option 2 — direct download from GitHub Releases

The `systemprompt-bridge` binaries ship under the `bridge-v*` tag series:

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

Run `systemprompt-bridge gui` and the **Setup wizard** opens automatically. Two steps:

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
brew uninstall systempromptio/tap/bridge     # Homebrew
rm /usr/local/bin/systemprompt-bridge        # manual install
```

To also wipe credentials and cache:

```bash
systemprompt-bridge uninstall --purge
```

This removes `~/.config/systemprompt/systemprompt-bridge.toml`, `~/Library/Caches/systemprompt-bridge/cache.json`, and `~/.config/systemprompt/agents.json`.

## Links

- [Desktop app overview](desktop-app.md)
- [Device auth modes](device-auth.md)
- [systemprompt.io](https://systemprompt.io/?utm_source=bridge-macos&utm_medium=install_doc)
- [Documentation](https://systemprompt.io/documentation/?utm_source=bridge-macos&utm_medium=install_doc)
- [systemprompt-template on GitHub](https://github.com/systempromptio/systemprompt-template)
- [LICENSE](../../LICENSE)
