# Cowork device authentication

The `systemprompt-bridge` binary (formerly `sp-cowork-auth` / `systemprompt-cowork`, renamed in v0.7.0) exchanges a local credential for a short-lived JWT + canonical identity header envelope that the Claude for Work client forwards to the gateway. Three modes are supported; pick the one that matches your device posture.

> **GUI shortcut.** If you have the [Desktop app](desktop-app.md) installed, the **Setup wizard** writes this config for you — paste a gateway URL + PAT (or pick session / mTLS) and the wizard persists `~/.config/systemprompt/systemprompt-bridge.toml`. Re-run any time from **Settings → Re-run setup**. The CLI flow below is for headless installs (CI, MDM rollouts) where you bypass the GUI.

## Modes at a glance

| Mode    | Credential source                            | Requires                                    | UI surface          |
|---------|----------------------------------------------|---------------------------------------------|---------------------|
| `pat`   | Bearer PAT (`sp-live-*.*`)                   | A PAT issued from `/admin/devices`          | `/admin/devices`    |
| `session` | Browser consent → 120 s one-shot code     | User logged into the dashboard              | `/cowork-auth/device-link` |
| `mtls`  | X.509 cert in OS keystore, fingerprint match | Cert enrolled via `systemprompt admin cowork enroll-cert` | (CLI only for now)  |

The server advertises what it supports at `GET /v1/auth/cowork/capabilities`.

## Config locations

| OS | Path |
|---|---|
| Linux / macOS | `~/.config/systemprompt/systemprompt-bridge.toml` |
| Windows | `%APPDATA%\systemprompt\systemprompt-bridge.toml` |

Override with `SP_BRIDGE_CONFIG=<path>`. Per-agent enable state lives alongside in `agents.json`.

## Quick setup

Pick one mode and write the config:

```toml
gateway_url = "https://your.gateway.example"

# Session flow (browser loopback on 127.0.0.1:8767)
[session]
enabled = true

# PAT flow (issue from /admin/devices, paste the one-shot secret here)
[pat]
token = "sp-live-abcdef012345.deadbeef..."

# mTLS flow (Linux PEM path; macOS/Windows read the keystore by label)
[mtls]
cert_path = "/etc/systemprompt/device.pem"
# macOS:
# cert_label = "systemprompt-device"
# Windows:
# cert_sha256 = "<64-hex-fingerprint>"
```

The helper probes providers in the order `mtls → session → pat`; the first one that returns `Ok` wins, and `NotConfigured` falls through. **mTLS-preferred is fail-closed**: a transient gateway failure on mTLS exits `10` instead of silently downgrading to PAT (v0.5.0+).

## Environment overrides

| Variable | Purpose |
|---|---|
| `SP_BRIDGE_CONFIG` | Path to `systemprompt-bridge.toml` |
| `SP_BRIDGE_GATEWAY_URL` | Gateway base URL (default `https://gateway.systemprompt.io`) |
| `SP_BRIDGE_PAT` | Inline PAT (overrides file-based `[pat]`) |
| `SP_BRIDGE_POLICY_PUBKEY` | Pinned manifest signing pubkey (overrides operator value) |
| `SP_BRIDGE_DEVICE_CERT_SHA256` | Pin a specific device cert by SHA-256 fingerprint |
| `SP_BRIDGE_LOG_FORMAT` | `json` for structured stderr logs |

The legacy `SP_COWORK_*` env vars are no longer read.

## Session flow walkthrough

1. User runs `systemprompt-bridge login --session` on their workstation.
2. Helper binds `127.0.0.1:8767` and opens `https://your.gateway/cowork-auth/device-link?redirect=http://127.0.0.1:8767/callback` in the default browser.
3. Dashboard renders a consent page (must be logged in) showing the loopback host. User clicks **Allow**.
4. Dashboard mints a 64-hex exchange code (SHA-256 hashed in `cowork_exchange_codes`, 120 s TTL, single-use) and redirects the browser to `http://127.0.0.1:8767/callback?code=...`.
5. Helper captures the code, POSTs to `/v1/auth/cowork/session`, and caches the returned JWT + headers.

The `/cowork-auth/device-link` consent page validates the redirect host — only `127.0.0.1` / `localhost` with a port are accepted.

## PAT flow walkthrough

1. User opens `/admin/devices`, clicks **Issue PAT**, names it (e.g. `laptop-m1`).
2. Dashboard returns the secret once (format: `sp-live-<12hex>.<52hex>`). Either paste it into the GUI Setup wizard, or run `systemprompt-bridge login sp-live-... --gateway https://...`.
3. The bridge sends `Authorization: Bearer <token>` to `/v1/auth/cowork/pat`.
4. Gateway verifies the secret against `user_api_keys.key_hash`, checks expiry/revocation, and returns the same JWT envelope as the session flow.

PAT revocation is immediate: **Revoke** on `/admin/devices` flips `revoked_at`, and the next verify call returns 401.

## mTLS flow walkthrough

1. Generate a device certificate on the workstation.
2. Compute its DER SHA-256 fingerprint as 64 lowercase hex chars.
3. Enroll it: `systemprompt admin cowork enroll-cert --user-id <UID> --fingerprint <SHA256> --label "laptop-m1"`.
4. On the workstation, point `[mtls].cert_path` at the PEM (Linux) or set `cert_label` / `cert_sha256` (macOS/Windows) so the bridge can locate the cert in the OS keystore.
5. The bridge reads the cert, recomputes the fingerprint, and POSTs it to `/v1/auth/cowork/mtls`. Gateway looks it up in `user_device_certs`, confirms it is not revoked, and returns the JWT envelope.

## Testing without a real device

Run `./demo/users/05-cowork-device-roundtrip.sh` — it mints a session code via the admin CLI, exchanges it against the live gateway, asserts the JWT and `x-sp-user-id` header come back, and verifies replay is rejected. No browser or helper binary required.

For a full browser round-trip from a `systemprompt-core` checkout:

```bash
systemprompt-bridge login --session --gateway http://localhost:8080
```

The consent page at `/cowork-auth/device-link` completes the loop.
