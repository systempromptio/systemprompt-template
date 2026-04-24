# Install cowork on macOS

`cowork` is the **client-side CLI** that runs on a developer workstation and talks to a running [systemprompt-gateway](../install/ghcr.md). It handles device auth (Claude for Work / MCP device flow) and code-worktree operations.

If you're looking to deploy the **server** (the gateway), see [../install/](../install/) instead — cowork is a separate product.

## Option 1 — Homebrew tap (recommended)

```bash
brew tap systempromptio/tap
brew install cowork
cowork --version
```

## Option 2 — direct download from GitHub Releases

Cowork bundles are published on the template repo under the `cowork-v*` tag series. Pick the matching architecture:

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -sSL -o cowork.tar.gz \
  https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.3.3/systemprompt-cowork-macos-aarch64.tar.gz
tar -xzf cowork.tar.gz
install -m 0755 systemprompt-cowork-macos-aarch64/cowork /usr/local/bin/cowork

# Intel Mac
curl -sSL -o cowork.zip \
  https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.3.3/systemprompt-cowork-macos-x64.zip
unzip cowork.zip
install -m 0755 systemprompt-cowork-macos-x64/cowork /usr/local/bin/cowork
```

Verify the SHA256:

```bash
curl -sSL -O https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.3.3/SHA256SUMS.cowork
shasum -a 256 -c SHA256SUMS.cowork --ignore-missing
```

Verify the cosign signature:

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-core/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature SHA256SUMS.cowork.sig \
  --certificate SHA256SUMS.cowork.pem \
  SHA256SUMS.cowork
```

## Configure against a running gateway

```bash
cowork config set gateway.url https://your-gateway.example.com
cowork login
```

See [device-auth.md](device-auth.md) for the auth-mode options.

## Uninstall

```bash
brew uninstall cowork            # Homebrew
rm /usr/local/bin/cowork         # manual install
```

## Links

- [systemprompt.io](https://systemprompt.io/?utm_source=cowork-macos&utm_medium=install_doc)
- [Documentation](https://systemprompt.io/documentation/?utm_source=cowork-macos&utm_medium=install_doc)
- [systemprompt-template on GitHub](https://github.com/systempromptio/systemprompt-template)
- [LICENSE](../../LICENSE)
