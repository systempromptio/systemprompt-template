# Windows bridge → Gateway → MiniMax Demo

End-to-end runbook for demoing Claude for Work on Windows routing every inference request through this gateway to MiniMax. Same governance pipeline as any other `/v1/messages` call — one row per request in the audit table.

**Audience:** engineer or SE running a live demo on a Windows laptop.
**Estimated time:** 15 minutes first-time, 5 minutes on a pre-staged machine.

---

## What's already wired in the `local` profile

No YAML edits needed. `.systemprompt/profiles/local/profile.yaml` ships with:

```yaml
providers:
  - name: minimax
    protocol: anthropic
    endpoint: https://api.minimax.io/anthropic/v1
    api_key_secret: minimax
    models:
      - id: MiniMax-M1
gateway:
  enabled: true
  default_provider: minimax
  routes:
    - model_pattern: "claude-*"
      provider: minimax
      upstream_model: MiniMax-M1
    - model_pattern: "*"
      provider: minimax
      upstream_model: MiniMax-M1
```

Provider endpoint and `api_key_secret` are declared once in the `providers` registry.

Every model string — including `claude-*` strings that Claude Desktop sends — is aliased onto `MiniMax-M1` via `upstream_model`. The client never sees the rewrite.

The `minimax` secret slot exists in `.systemprompt/profiles/local/secrets.json`; you just need to make sure it holds a real key.

The Windows helper is **`systemprompt-bridge.exe`** (renamed from `sp-cowork-auth` / `systemprompt-cowork` in v0.7.0). It runs both as a credential-helper CLI (Claude Desktop's Inference slot) and as a Desktop GUI (`systemprompt-bridge gui`). Config lives in `%APPDATA%\systemprompt\systemprompt-bridge.toml`. PATs are issued via the web UI at `/admin/devices`, not a CLI subcommand.

---

## 1. Prepare the gateway host (WSL / Linux / VM)

```bash
# 1. Confirm the MiniMax key is real (not a placeholder)
jq -r '.minimax' .systemprompt/profiles/local/secrets.json

# If empty or placeholder:
systemprompt cloud secrets set minimax <your-minimax-api-key>

# 2. Build + start
just build && just start

# 3. Sanity: auth surface is up
curl -s http://localhost:8080/v1/auth/cowork/capabilities
# expect: {"modes":[...]}

# 4. Full PAT → JWT → gateway roundtrip (automated)
./demo/users/05-cowork-device-roundtrip.sh
# expect: 5/5 ✓
```

If step 4 fails, stop — the server build is broken and no Windows flow will work.

## 2. Make the host reachable from Windows

- **Windows is the WSL host.** `http://localhost:8080` works from Windows directly via WSL2 localhost forwarding.
- **Separate machine.** Bind the server to a LAN IP, open port 8080, or run a tunnel (e.g. `cloudflared`, `ngrok`). Test from the Windows box: `curl http://<host>:8080/v1/auth/cowork/capabilities`.

## 3. Issue a PAT for the demo user

1. Browser → `http://<host>:8080/admin/login` and sign in.
2. `http://<host>:8080/admin/devices` → **Personal access tokens** → name it `cowork-demo` → **Issue PAT**.
3. Copy the `sp-live-...` value. **This is the only time it's shown.**

---

## 4. Install the bridge on the Windows machine

Latest Windows build is in **[bridge-v0.10.0](https://github.com/systempromptio/systemprompt-template/releases/tag/bridge-v0.10.0)** — asset `systemprompt-bridge-x86_64-pc-windows-msvc.exe`.

Easiest: `scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket && scoop install bridge`. Manual:

```powershell
$dir = "C:\Program Files\systemprompt"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

Invoke-WebRequest `
  -Uri "https://github.com/systempromptio/systemprompt-template/releases/download/bridge-v0.10.0/systemprompt-bridge-x86_64-pc-windows-msvc.exe" `
  -OutFile "$dir\systemprompt-bridge.exe"

[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$dir", "User")
```

Open a new terminal so the PATH change takes effect. Windows SmartScreen will flag the unsigned binary on first run → **More info** → **Run anyway**. Do this once before the demo.

## 5. Store the PAT on the Windows box

**GUI path** (recommended for live demos — visible feedback):

```powershell
systemprompt-bridge gui
```

The Setup wizard opens. Paste the gateway URL and PAT, then enable Claude Desktop on the agents step. The status pill goes green when the proxy is up and the PAT is verified. Skip to step 7.

**CLI path** (scriptable):

```powershell
systemprompt-bridge login sp-live-PASTE_YOUR_SECRET --gateway http://<host>:8080
systemprompt-bridge status
```

`login` writes:

- `%APPDATA%\systemprompt\systemprompt-bridge.toml` (gateway URL + PAT, user-scoped NTFS ACL)

Env-only alternative (no secret on disk, good for CI):

```powershell
$env:SP_BRIDGE_PAT = "sp-live-..."
$env:SP_BRIDGE_GATEWAY_URL = "http://<host>:8080"
```

## 6. Verify the bridge issues a JWT

```powershell
systemprompt-bridge
```

Expect one JSON object on stdout:

```json
{"token":"eyJ0eXAi...","ttl":3600,"headers":{"x-user-id":"...","x-session-id":"...","x-trace-id":"...","x-tenant-id":"...","x-client-id":"sp_cowork","x-call-source":"cowork","x-policy-version":"unversioned"}}
```

Prove the JWT is accepted downstream before Claude Desktop is in the loop:

```powershell
$env:TOKEN = (systemprompt-bridge | ConvertFrom-Json).token
curl -H "Authorization: Bearer $env:TOKEN" http://<host>:8080/api/v1/core/oauth/userinfo
# expect: your user profile JSON
```

## 7. Point Claude for Work at the gateway

The simplest path: in the bridge Desktop GUI, open the **Agents** tab and click **Enable** on the Claude Desktop card, then **Generate profile** and **Install profile**. The bridge writes the credential-helper config and the inference base URL into Claude Desktop's managed profile in one click.

Manual fallback — Claude Desktop → **Enterprise → Settings → Inference**:

- **Credential helper script**: `C:\Program Files\systemprompt\systemprompt-bridge.exe`
- **API base URL**: `http://127.0.0.1:48217` (the local proxy; the bridge swaps the loopback secret for a fresh JWT and forwards to the gateway URL configured in step 5)

---

## 8. Run the demo

1. In Claude Desktop, send a prompt. Model name is irrelevant — the catch-all route sends every request to `MiniMax-M1`. The client sees a normal Anthropic-shaped response; the upstream is MiniMax.
2. On the gateway, show the audit row land live:

   ```bash
   systemprompt infra logs request list --limit 5
   systemprompt infra logs audit <request-id> --full
   systemprompt analytics costs
   systemprompt analytics requests
   ```

   Point at: `user_id`, `trace_id`, requested model (e.g. `claude-*`), upstream provider (`minimax`), tokens, cost in microdollars, latency.

3. (Optional kicker) Send a prompt containing a fake secret and show it denied in `logs request list --status failed`. Same governance pipeline applies to `/v1/messages`, not just tool calls.

---

## Gotchas — pre-flight 10 minutes before

- **Confirm the exact Windows asset filename** on the release page (currently `systemprompt-bridge-x86_64-pc-windows-msvc.exe`). What you install locally is what Claude Desktop's helper-script path points at.
- **First-run SmartScreen** — trigger it once yourself so it doesn't interrupt the live demo.
- **Clock skew** breaks JWT verification. Sync the Windows clock.
- **Cached JWT** — after re-issuing the PAT, delete `%LOCALAPPDATA%\systemprompt-bridge\cache.json` so the bridge fetches fresh. (Or: GUI → Settings → Re-run setup.)
- **Diagnostics** — if anything goes sideways, the activity drawer's **Help & Support → Export diagnostic bundle** drops a redacted zip on the Desktop with logs, activity JSONL, crash dumps, and config.
- **The catch-all is MiniMax for *everything*.** Calling it out on slide one prevents the "why did my `claude-opus` request go to MiniMax?" question.
- **PAT scoping.** The user who issued the PAT must have permission for the gateway route. Step 6's `userinfo` call is the cheapest way to verify.

---

## Cleanup after the demo

```powershell
systemprompt-bridge logout                                          # removes PAT + strips [pat]
Remove-Item "$env:LOCALAPPDATA\systemprompt-bridge\cache.json" -ErrorAction Ignore
# or wipe everything: systemprompt-bridge uninstall --purge
```

Revoke the PAT from `/admin/devices` on the gateway.

---

## Reference

- Profile config: `.systemprompt/profiles/local/profile.yaml`
- Secrets: `.systemprompt/profiles/local/secrets.json`
- Gateway capabilities probe: `GET /v1/auth/cowork/capabilities`
- PAT exchange: `POST /v1/auth/cowork/pat` with `Authorization: Bearer sp-live-...`
- Full auth / flow spec: `docs/manual-cowork-test.md`
- Device cert (mTLS) enrollment: [`device-auth.md`](device-auth.md)
