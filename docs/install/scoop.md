# Install the gateway via Scoop (Windows)

Installs the `systemprompt-gateway` server natively on Windows using [Scoop](https://scoop.sh) (available from gateway `v0.5.0`). For the bridge client on Windows, see [../cowork/scoop.md](../cowork/scoop.md).

Bucket: [`systempromptio/scoop-bucket`](https://github.com/systempromptio/scoop-bucket).

## Install Scoop (once)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

## Install the gateway

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install gateway
systemprompt --version
```

Installs `systemprompt.exe` and `systemprompt-mcp-agent.exe` onto your `PATH`, with `services/` and `migrations/` alongside in the package directory.

## Run

The gateway needs Postgres. Point `DATABASE_URL` at any reachable instance and set at least one provider key:

```powershell
$env:DATABASE_URL = "postgres://user:pw@localhost:5432/systemprompt"
$env:ANTHROPIC_API_KEY = "sk-ant-..."
systemprompt infra db migrate
systemprompt infra services start --foreground
```

## Upgrade

```powershell
scoop update gateway
```

Docs: https://systemprompt.io/documentation/?utm_source=scoop&utm_medium=install_doc
