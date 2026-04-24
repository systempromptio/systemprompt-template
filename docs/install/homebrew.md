# Install the gateway via Homebrew

Installs the `systemprompt-gateway` server on macOS (or Linuxbrew). For the cowork client CLI, see [../cowork/install-macos.md](../cowork/install-macos.md).

Uses the [`systempromptio/tap`](https://github.com/systempromptio/homebrew-tap) tap — faster iteration than Homebrew core, and avoids pinning our release cadence to homebrew/core review queues. The compiled binary is `MIT AND BUSL-1.1` (template MIT + linked BSL-1.1 core).

## Install

```bash
brew tap systempromptio/tap
brew install gateway
```

## Upgrade

```bash
brew update
brew upgrade gateway
```

## Run as a service

```bash
brew services start gateway
```

Logs: `~/Library/Logs/Homebrew/gateway.log` (macOS) or `$(brew --prefix)/var/log/gateway.log` (Linuxbrew).

Stop:

```bash
brew services stop gateway
```

## Configuration

The formula installs `systemprompt`, `systemprompt-mcp-agent`, and `systemprompt-mcp-marketplace` to `$(brew --prefix)/bin`.

Before starting, export required env vars or set them in `$(brew --prefix)/etc/systemprompt.env`:

```
DATABASE_URL=postgres://user:pw@host:5432/systemprompt
ANTHROPIC_API_KEY=sk-ant-...
```

Start Postgres via Homebrew too if you don't already run one:

```bash
brew install postgresql@16
brew services start postgresql@16
createdb systemprompt
```

Docs: https://systemprompt.io/documentation/?utm_source=homebrew&utm_medium=install_doc
