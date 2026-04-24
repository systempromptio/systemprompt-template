# Cowork device authentication

The `sp-cowork-auth` helper (ships with `systemprompt-core`) exchanges a local credential for a short-lived JWT + canonical identity header envelope that the Claude for Work client forwards to the gateway. Three modes are supported; pick the one that matches your device posture.

## Modes at a glance

| Mode    | Credential source                            | Requires                                    | UI surface          |
|---------|----------------------------------------------|---------------------------------------------|---------------------|
| `pat`   | Bearer PAT (`sp-live-*.*`)                   | A PAT issued from `/admin/devices`          | `/admin/devices`    |
| `session` | Browser consent → 120 s one-shot code     | User logged into the dashboard              | `/cowork-auth/device-link` |
| `mtls`  | X.509 cert in OS keystore, fingerprint match | Cert enrolled via `systemprompt admin cowork enroll-cert` | (CLI only for now)  |

The server advertises what it supports at `GET /v1/auth/cowork/capabilities`.

## Quick setup

Pick one mode and configure `~/.config/systemprompt/cowork-auth.toml`:

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

The helper probes providers in the order `mtls → session → pat`; the first one that returns `Ok` wins, and `NotConfigured` falls through.

## Session flow walkthrough

1. User runs `sp-cowork-auth session` on their workstation.
2. Helper binds `127.0.0.1:8767` and opens `https://your.gateway/cowork-auth/device-link?redirect=http://127.0.0.1:8767/callback` in the default browser.
3. Dashboard renders a consent page (must be logged in) showing the loopback host. User clicks **Allow**.
4. Dashboard mints a 64-hex exchange code (SHA-256 hashed in `cowork_exchange_codes`, 120 s TTL, single-use) and redirects the browser to `http://127.0.0.1:8767/callback?code=...`.
5. Helper captures the code, POSTs to `/v1/auth/cowork/session`, and caches the returned JWT + headers.

The `/cowork-auth/device-link` consent page validates the redirect host — only `127.0.0.1` / `localhost` with a port are accepted.

## PAT flow walkthrough

1. User opens `/admin/devices`, clicks **Issue PAT**, names it (e.g. `laptop-m1`).
2. Dashboard returns the secret once (format: `sp-live-<12hex>.<52hex>`). Copy it into `~/.config/systemprompt/cowork-auth.toml` under `[pat].token`.
3. `sp-cowork-auth pat` sends `Authorization: Bearer <token>` to `/v1/auth/cowork/pat`.
4. Gateway verifies the secret against `user_api_keys.key_hash`, checks expiry/revocation, and returns the same JWT envelope as the session flow.

PAT revocation is immediate: **Revoke** on `/admin/devices` flips `revoked_at`, and the next verify call returns 401.

## mTLS flow walkthrough

1. Generate a device certificate on the workstation.
2. Compute its DER SHA-256 fingerprint as 64 lowercase hex chars.
3. Enroll it: `systemprompt admin cowork enroll-cert --user-id <UID> --fingerprint <SHA256> --label "laptop-m1"`.
4. On the workstation, point `[mtls].cert_path` at the PEM (Linux) or set `cert_label` / `cert_sha256` (macOS/Windows) so the helper can locate the cert in the OS keystore.
5. `sp-cowork-auth mtls` reads the cert, recomputes the fingerprint, and POSTs it to `/v1/auth/cowork/mtls`. Gateway looks it up in `user_device_certs`, confirms it is not revoked, and returns the JWT envelope.

## Testing without a real device

Run `./demo/users/05-cowork-device-roundtrip.sh` — it mints a session code via the admin CLI, exchanges it against the live gateway, asserts the JWT and `x-sp-user-id` header come back, and verifies replay is rejected. No browser or helper binary required.

For a full browser round-trip, run `sp-cowork-auth session --gateway http://localhost:8080` from the core checkout; the consent page at `/cowork-auth/device-link` completes the loop.
