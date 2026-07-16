# Install the bridge credential helper

The `systemprompt-bridge` binary is the **Credential helper script** slot in Claude for Work. It turns a PAT into a short-lived JWT that Claude Desktop merges into every inference request routed at this binary. Download the prebuilt macOS, Windows, or Linux binary from [systempromptio/systemprompt-core releases](https://github.com/systempromptio/systemprompt-core/releases/tag/bridge-v0.10.0).

`systemprompt-bridge` is a standalone ~2.4 MB Rust binary (no `tokio`, no `sqlx`, no `axum`) that trades a lower-privilege credential for a short-lived JWT. Progressive capability ladder (mTLS → dashboard session → PAT) mounted under `/v1/gateway/auth/bridge/`:

- `POST /pat` — `Authorization: Bearer <pat>` → `{token, ttl, headers}` with a fresh JWT and the canonical identity header map (`x-user-id`, `x-session-id`, `x-trace-id`, `x-client-id`, `x-tenant-id`, `x-policy-version`, `x-call-source`).
- `POST /session` — `501` (dashboard-cookie exchange not yet wired).
- `POST /mtls` — `501` (device-cert exchange not yet wired).
- `GET /capabilities` — `{"modes":["pat"]}`; probes advertise which exchange modes this deployment accepts.

The helper writes the signed JWT + expiry to the OS cache dir with mode `0600`. Stdout contract is exactly one JSON object; all diagnostics go to stderr. Released out-of-band as `bridge-v*` tags.

Current release: **[bridge-v0.10.0](https://github.com/systempromptio/systemprompt-core/releases/tag/bridge-v0.10.0)** — Linux x86_64, Windows x86_64 (MSVC ABI), macOS aarch64 (cosign-signed).

## 1. Download

**Linux x86_64**

```bash
curl -fsSL -o /usr/local/bin/systemprompt-bridge \
  https://github.com/systempromptio/systemprompt-core/releases/download/bridge-v0.10.0/systemprompt-bridge-x86_64-unknown-linux-gnu
chmod +x /usr/local/bin/systemprompt-bridge
curl -fsSL -O https://github.com/systempromptio/systemprompt-core/releases/download/bridge-v0.10.0/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

**Windows x86_64** (PowerShell as Administrator):

```powershell
$dir = "C:\Program Files\systemprompt"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Invoke-WebRequest `
  -Uri "https://github.com/systempromptio/systemprompt-core/releases/download/bridge-v0.10.0/systemprompt-bridge-x86_64-pc-windows-msvc.exe" `
  -OutFile "$dir\systemprompt-bridge.exe"
[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$dir", "User")
```

Windows Smart Screen will flag the unsigned binary on first run → "More info" → "Run anyway".

**macOS** (source build):

```bash
git clone https://github.com/systempromptio/systemprompt-core.git
cd systemprompt-core
cargo build --manifest-path bin/bridge/Cargo.toml --release \
  --target "$(rustc -vV | awk '/host:/ {print $2}')"
sudo install -m 755 \
  "bin/bridge/target/$(rustc -vV | awk '/host:/ {print $2}')/release/systemprompt-bridge" \
  /usr/local/bin/
```

## 2. Configure

Linux/macOS: `~/.config/systemprompt/systemprompt-bridge.toml`
Windows: `%APPDATA%\systemprompt\systemprompt-bridge.toml`

```toml
[gateway]
url = "http://localhost:8080"   # for the local-trial template; swap to your production host

[pat]
token = "sp-live-your-personal-access-token"
```

Issue a PAT from the running binary with `systemprompt admin users pat issue <user-id> --name bridge-laptop`. Absent config sections are silently skipped. Dev overrides: `SP_BRIDGE_GATEWAY_URL`, `SP_BRIDGE_PAT`.

## 3. Verify

```bash
systemprompt-bridge           # prints exactly one JSON {token, ttl, headers}
systemprompt-bridge --check   # exits 0 if a token can be issued
```

Diagnostics go to stderr only. The stdout JSON matches Anthropic's `inferenceCredentialHelper` contract byte-for-byte.

## 4. Point Claude Desktop at it

In Claude Desktop **Enterprise → Settings → Inference**:

- **Credential helper script**: `/usr/local/bin/systemprompt-bridge` (or `C:\Program Files\systemprompt\systemprompt-bridge.exe`).
- **API base URL**: the `gateway.url` from your TOML.

Every Claude Desktop request now lands a row in `ai_requests` with `user_id`, `tenant_id`, `session_id`, `trace_id`, tokens, cost, and latency — identical governance to every other tool call. Run `systemprompt infra logs audit <request-id> --full` after a prompt to see the trace end-to-end.

## 5. (Optional) Install the `org-plugins/` sync agent

The same binary manages the bridge's signed plugin / managed-MCP mount:

```bash
systemprompt-bridge install     # register launchd (macOS) / scheduled task (Windows) / systemd --user (Linux)
systemprompt-bridge sync        # pull signed plugin manifest + allowlist now
systemprompt-bridge validate    # verify the ed25519 signature
systemprompt-bridge uninstall   # remove
```

Mount targets: `/Library/Application Support/Claude/org-plugins/` (macOS), `C:\ProgramData\Claude\org-plugins\` (Windows), `${XDG_DATA_HOME:-$HOME/.local/share}/Claude/org-plugins/` (Linux).
