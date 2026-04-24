# Install via APT (Debian / Ubuntu)

> **Status: deferred.** APT publishing is parked until a Linux-sysadmin user asks for it.
> The `apt.yml` workflow exists but is triggered manually only.
> In the meantime, use [binary](binary.md), [GHCR](ghcr.md), or [Helm](helm.md).

Packages are published to [`deb.systemprompt.io`](https://deb.systemprompt.io) — a GPG-signed APT repo for `amd64` and `arm64` on Debian 12+ and Ubuntu 22.04+.

## Add the repo

```bash
curl -fsSL https://deb.systemprompt.io/gpg.key \
  | sudo gpg --dearmor -o /usr/share/keyrings/systemprompt.gpg

echo "deb [signed-by=/usr/share/keyrings/systemprompt.gpg] https://deb.systemprompt.io stable main" \
  | sudo tee /etc/apt/sources.list.d/systemprompt.list

sudo apt-get update
```

## Install

```bash
sudo apt-get install systemprompt
```

## Configure

Edit `/etc/systemprompt/systemprompt.env`:

```
DATABASE_URL=postgres://systemprompt:pw@localhost:5432/systemprompt
ANTHROPIC_API_KEY=sk-ant-...
HOST=0.0.0.0
PORT=8080
```

Make sure you have Postgres running locally or reachable:

```bash
sudo apt-get install postgresql
sudo -u postgres createuser --pwprompt systemprompt
sudo -u postgres createdb -O systemprompt systemprompt
```

## Start

```bash
sudo systemctl enable --now systemprompt
sudo systemctl status systemprompt
journalctl -u systemprompt -f
```

## Upgrade

```bash
sudo apt-get update && sudo apt-get upgrade systemprompt
```

## Package layout

| Path | Purpose |
|---|---|
| `/usr/bin/systemprompt` | Gateway binary |
| `/usr/bin/systemprompt-mcp-{agent,marketplace}` | MCP servers |
| `/etc/systemprompt/services/` | YAML configuration tree |
| `/etc/systemprompt/systemprompt.env` | Environment overrides (edit this) |
| `/usr/share/systemprompt/migrations/` | DB migrations |
| `/lib/systemd/system/systemprompt.service` | systemd unit |

Docs: https://systemprompt.io/documentation/?utm_source=apt&utm_medium=install_doc
