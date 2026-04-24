# Install via DNF / YUM (RHEL, Rocky, Fedora, Amazon Linux)

> **Status: deferred.** RPM publishing is parked until a RHEL/Fedora user asks for it.
> The `rpm.yml` workflow exists but is triggered manually only.
> In the meantime, use [binary](binary.md), [GHCR](ghcr.md), or [Helm](helm.md).

Packages are published to [`rpm.systemprompt.io`](https://rpm.systemprompt.io) — a GPG-signed RPM repo for `x86_64` and `aarch64` on RHEL 9 / Rocky 9 / Fedora 40+.

## Add the repo

```bash
sudo curl -fsSL -o /etc/yum.repos.d/systemprompt.repo \
  https://rpm.systemprompt.io/systemprompt.repo

sudo rpm --import https://rpm.systemprompt.io/gpg.key
```

## Install

```bash
sudo dnf install systemprompt
```

(On older systems: `sudo yum install systemprompt`.)

## Configure

Edit `/etc/systemprompt/systemprompt.env`:

```
DATABASE_URL=postgres://systemprompt:pw@localhost:5432/systemprompt
ANTHROPIC_API_KEY=sk-ant-...
HOST=0.0.0.0
PORT=8080
```

Install and start Postgres if needed:

```bash
sudo dnf install postgresql-server postgresql-contrib
sudo postgresql-setup --initdb
sudo systemctl enable --now postgresql
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
sudo dnf upgrade systemprompt
```

## Package layout

Same as the [APT package](apt.md) — binaries in `/usr/bin`, config in `/etc/systemprompt/`, unit in `/usr/lib/systemd/system/systemprompt.service`.

Docs: https://systemprompt.io/documentation/?utm_source=rpm&utm_medium=install_doc
