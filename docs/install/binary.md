# Install the gateway binary (from GitHub Releases)

Installs the `systemprompt-gateway` server binary. For the cowork client CLI, see [../cowork/](../cowork/).

Single-shell installer:

```bash
curl -sSL https://get.systemprompt.io | sh
```

This detects your OS + arch, downloads the signed tarball, verifies SHA256, and installs to `/usr/local/bin` (root) or `~/.local/bin` (user).

## Flags

```bash
# Pin a specific version
curl -sSL https://get.systemprompt.io | sh -s -- --version v0.2.2

# Install to a custom prefix
curl -sSL https://get.systemprompt.io | sh -s -- --prefix /opt/systemprompt

# Additionally verify the cosign signature (requires `cosign` in PATH)
curl -sSL https://get.systemprompt.io | sh -s -- --verify
```

## Manual download

Pick your tarball from [Releases](https://github.com/systempromptio/systemprompt-template/releases/latest):

| OS | Arch | Asset |
|---|---|---|
| Linux | x86_64 | `systemprompt-<version>-linux-amd64.tar.gz` |
| Linux | arm64 | `systemprompt-<version>-linux-arm64.tar.gz` |
| macOS | Intel | `systemprompt-<version>-darwin-amd64.tar.gz` |
| macOS | Apple Silicon | `systemprompt-<version>-darwin-arm64.tar.gz` |
| Windows | x86_64 | `systemprompt-<version>-windows-amd64.zip` |

```bash
# Verify SHA256
curl -LO https://github.com/systempromptio/systemprompt-template/releases/download/v0.2.2/SHA256SUMS.gateway
grep systemprompt-0.2.2-linux-amd64.tar.gz SHA256SUMS.gateway | sha256sum -c -

# Extract
tar -xzf systemprompt-0.2.2-linux-amd64.tar.gz
cd systemprompt-0.2.2-linux-amd64
./systemprompt --version
```

## Verify signature

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-template/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature SHA256SUMS.gateway.sig \
  --certificate SHA256SUMS.gateway.pem \
  SHA256SUMS.gateway
```

## What's in the tarball

| Path | Purpose |
|---|---|
| `systemprompt` | Gateway binary |
| `systemprompt-mcp-agent` | MCP agent server |
| `systemprompt-mcp-marketplace` | MCP marketplace server |
| `services/` | YAML configuration tree |
| `migrations/` | Database migrations |
| `web/` | Bundled web assets |

## Run

You need a Postgres database and at least one AI provider key.

```bash
export DATABASE_URL=postgres://user:pw@host:5432/systemprompt
export ANTHROPIC_API_KEY=sk-ant-...
systemprompt
```

Docs: https://systemprompt.io/documentation/?utm_source=binary&utm_medium=install_doc
