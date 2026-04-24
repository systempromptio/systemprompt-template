# Install via Winget (Windows 11)

> **Status: deferred.** Winget publishing is parked until there's Windows enterprise demand.
> The `winget.yml` workflow exists but is triggered manually only.
> In the meantime, use [Scoop](scoop.md) or [binary](binary.md) on Windows.

[Winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) is the official Windows Package Manager, pre-installed on Windows 11.

## Install

```powershell
winget install systemprompt.gateway
```

## Upgrade

```powershell
winget upgrade systemprompt.gateway
```

## Uninstall

```powershell
winget uninstall systemprompt.gateway
```

## Run as a Windows service

Winget drops the binaries into your `%LOCALAPPDATA%\Microsoft\WinGet\Packages\...`. To run as a service, see [scoop.md](scoop.md#run-as-a-windows-service) for the `nssm` pattern — same approach works with the winget-installed path.

## Configure

Set env vars either per-user:

```powershell
[Environment]::SetEnvironmentVariable("DATABASE_URL","postgres://...","User")
[Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY","sk-ant-...","User")
```

Or system-wide (requires admin):

```powershell
[Environment]::SetEnvironmentVariable("DATABASE_URL","postgres://...","Machine")
```

Docs: https://systemprompt.io/documentation/?utm_source=winget&utm_medium=install_doc
