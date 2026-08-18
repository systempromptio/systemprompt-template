# Bug: Loopback secret rotation leaves host profiles stale → persistent `403 bad loopback secret`, with misleading remediation

**Component:** `systemprompt-bridge` (core `bin/bridge`) — local inference proxy + host integration
**Affected white-label:** astound-bridge 0.14.0 (commit `bad1b596`, windows x86_64)
**Severity:** High — blocks Cowork/Claude sign-in entirely; advertised fix does not work
**Status:** Open
**Reporter:** Ed (ed@systemprompt.io), 2026-06-25

---

## Summary

When the bridge's loopback secret (`bridge-loopback.key`) is (re)minted — on first run, fresh
login / "runtime config swapped", or any time the key file is absent — the bridge does **not**
rewrite the host profiles it previously installed (Cowork / Claude Desktop). Those profiles still
carry the **old** secret as their `api_key`, so the client presents a stale token to the proxy and
is rejected with `403 forbidden: bad loopback secret` on every request.

The defect is compounded by **incorrect remediation guidance**: the proxy log, the bridge doctor
check, and the user-facing gateway sign-in error all tell the user to **restart Claude Desktop**.
Restarting the client only re-reads the same stale profile, so it can never resolve a secret
mismatch. The user-facing error additionally misattributes the cause to the upstream provider
("expired key or wrong region"), sending users to debug the gateway/provider when the problem is
entirely local.

## Impact

- Cowork/Claude cannot authenticate to the local proxy; all `/v1/messages` calls 403.
- The advertised fix ("restart Claude Desktop") is a no-op for this failure mode, so users loop on
  it indefinitely (confirmed: reporter restarted Claude with no effect).
- The user-facing copy points at the wrong layer (provider credential / region / expired key),
  maximizing time-to-diagnosis.

## Confirmed live evidence (diagnostics bundle `…073800Z.zip`)

A second bundle captured after the user restarted the client confirms the mechanism with
fingerprints — the proxy logs an 8-char SHA of both the presented and expected secret:

| fingerprint | value | source |
|---|---|---|
| `expected_fp` | **`2d738a69`** | proxy's live key file `…\astound\bridge-loopback.key` |
| `presented_fp` | **`a6ee3c83`** | secret the Claude/Cowork client sends (`presented_len=43` — a valid 32-byte secret, just the **wrong** one) |

Key file provenance (`bridge.2026-06-24.log`):

```
INFO loopback secret file not present; will mint on proxy_init  path=…\astound\bridge-loopback.key
INFO minted fresh loopback secret; restart Claude Desktop to pick it up  path=…\astound\bridge-loopback.key fp=2d738a69
```

So the proxy's key (`2d738a69`) was minted on 2026-06-24, but the client is still presenting an
**older** secret (`a6ee3c83`) that predates that mint — i.e. a profile written before this bridge's
key existed (almost certainly a leftover from a prior install/uninstall cycle) was never rewritten.
This is the drift the report describes, observed directly.

Three facts nail it:

1. **`a6ee3c83` never changes** across the whole log — including across two *different* Claude
   Electron builds hitting the proxy (`Claude/1.11847.5 Electron/41.6.1` **and**
   `Claude/1.15200.0 Electron/42.4.0`) and across the user's restart between the two bundles.
   Restarting / upgrading the client provably does not update the secret.
2. `activity.jsonl` shows `"opened host Claude Desktop"` (id 13) but **no** profile
   generate/apply event anywhere — opening a host (or restarting it) does not rewrite the secret;
   only an explicit profile-generate does, and it never ran.
3. The proxy emits the misleading `"restart Claude Desktop"` remediation into both the log and
   `activity.jsonl` on every rejection (ids 14–27), steering the user to the one action that can't
   work.

Empty-secret callers also appear (`ua=Bun/1.3.14`, `presented_fp=<empty>`, `presented_len=0`) — a
sidecar making unauthenticated probes; not the primary failure but additional 403 noise.

## Observed

User-facing error (Cowork gateway sign-in probe):

```
Couldn't sign in to Gateway
The provider rejected the credentials IT configured. This usually means an expired key or wrong region.
message:      Gateway rejected the configured credential (HTTP 403).
httpStatus:   403
requestUrl:   http://127.0.0.1:48217/v1/messages
probedModel:  claude-haiku-4-5-20251001
responseBody: forbidden: bad loopback secret
endpoint:     http://127.0.0.1:48217/
checkedAt:    2026-06-25T07:33:02Z
```

Bridge log around the triggering login (from `astound-bridge-diagnostics-20260625T071334Z.zip`):

```
INFO login: PAT and config written config_file=…\astound\astound-bridge.toml
INFO runtime config swapped
INFO first-run trust-on-first-use: fetching manifest pubkey from gateway
…
WARN proxy: listening on localhost:48217
```

## Root cause

The loopback secret has **two independent consumers that are never reconciled**:

1. **Proxy verifier** — `proxy/server.rs:63` calls `secret::proxy_init()`, which mints a new secret
   when `bridge-loopback.key` is missing/empty (`proxy/secret.rs`, `mint()` →
   `"minted fresh loopback secret; restart Claude Desktop to pick it up"`). The proxy then verifies
   every request's `Authorization: Bearer <token>` against this value
   (`proxy/dispatch/auth.rs::verify_loopback_secret`).

2. **Host profile writer** — `gui/hosts/handlers.rs::generate_profile_for` (line 446) reads the
   secret via `secret::for_profile()` and bakes it into the host config as `api_key`
   (`ProfileGenInputs { api_key: loopback_secret, … }`, line 480). This runs **only** when a host
   profile is generated/applied (`on_profile_generate_requested`, line 178) — a manual/GUI action,
   or `astound-bridge install --apply`.

There is **no path that re-runs (2) when (1) changes the secret.** So any event that rotates the key
(first run, re-mint after the key file is removed, fresh login / runtime-config swap) leaves every
already-installed host profile pinned to the previous secret. The proxy and the client now disagree
permanently until a human re-applies the host profile.

`catalog/plugins.rs` guarantees the manifest-hash and byte-serving paths can't drift because they
share one in-memory bundle. The loopback secret has **no equivalent single-source guarantee across
the verifier and the installed host profiles** — that asymmetry is the bug.

## Misleading remediation (three places)

1. `proxy/secret.rs` mint log: `"minted fresh loopback secret; restart Claude Desktop to pick it up"`
2. `proxy/dispatch/auth.rs::verify_loopback_secret` rejection log:
   `"reject: bad loopback secret — restart Claude Desktop to pick up the current secret"`
3. `cli/doctor/auth.rs:153` loopback-secret check hint: `"… restart Claude Desktop …"`

All three assume the client can pick up the *current* secret by restarting. It cannot — the client
only re-reads the profile the bridge wrote, which still holds the old secret. The correct remediation
is **regenerate the host profile from the bridge** (`on_profile_generate_requested` /
`install --apply`), *then* restart the client.

Separately, the **user-facing** gateway sign-in copy
("The provider rejected the credentials IT configured … expired key or wrong region") is wrong for
`responseBody: forbidden: bad loopback secret` — that response is the **local proxy** rejecting a
**local** secret mismatch, unrelated to the upstream provider, key expiry, or region.

## Steps to reproduce

1. Have a host (Cowork/Claude) whose profile already carries a loopback secret S1 (e.g. from a prior
   bridge install). **Observed here:** client presents `a6ee3c83`.
2. (Re)install / first-run the bridge so `bridge-loopback.key` is absent and `proxy_init()` mints a
   fresh secret S2 ≠ S1 → proxy now verifies against S2. **Observed here:** key minted 2026-06-24,
   `fp=2d738a69`. Nothing rewrites the host profile, which still holds S1.
3. Restart Claude/Cowork (do **not** re-apply the host profile from the bridge). **Observed:** even
   restarting *and* upgrading the client leaves the presented secret at S1 (`a6ee3c83`).
4. Sign in / send a request.

**Expected:** sign-in succeeds, or the error tells the user to re-apply the host profile from the bridge.
**Actual:** `403 forbidden: bad loopback secret`; the error and logs tell the user to restart the
client, which never resolves it.

## Workaround (current)

In the **Astound Bridge** app, regenerate the host profile for Cowork/Claude
(`on_profile_generate_requested` — "Generate / Re-apply / Reconnect"), **or** run
`astound-bridge install --apply`. *Then* restart the client so it re-reads the rewritten profile.
Restarting the client alone does nothing.

## Suggested fixes

1. **Reconcile on rotation (primary).** When `proxy_init()` mints a new secret, re-run host profile
   generation for all installed hosts so their `api_key` matches the live secret. Equivalently, run
   the host re-apply automatically on proxy start / after login when the key changed. The verifier
   and the installed profiles must share a single source of truth, mirroring the
   `catalog/plugins.rs` "cannot drift" guarantee.
2. **Avoid needless rotation.** Confirm login / "runtime config swapped" does not delete or bypass an
   existing `bridge-loopback.key`; preserve the key across logins so installed profiles stay valid.
3. **Fix remediation copy** in all three internal sites (`secret.rs`, `dispatch/auth.rs`,
   `doctor/auth.rs`): on `bad loopback secret`, instruct "re-apply the host profile from the bridge
   (or `install --apply`), then restart the client" — not "restart Claude Desktop".
4. **Fix user-facing error** for `responseBody: forbidden: bad loopback secret`: surface it as a
   local proxy/secret mismatch with the re-apply instruction, not as a provider credential / region /
   expiry problem.
5. **Doctor upgrade.** `cli/doctor/auth.rs::check_loopback_secret` currently only checks the key is
   *present*. Have it compare the key against the secret embedded in each installed host profile and
   fail (with the re-apply remediation) when they diverge — that turns this from a silent 403 into a
   diagnosable check.

## Evidence / references

- Diagnostics bundles: `astound-bridge-diagnostics-20260625T071334Z.zip` and
  `…20260625T073800Z.zip` — the second (`bridge.2026-06-25.log`, `bridge.2026-06-24.log`,
  `activity.jsonl`) carries the fingerprint evidence (`expected_fp=2d738a69`,
  `presented_fp=a6ee3c83`) and the 2026-06-24 mint line.
- Core source (read-only submodule `systemprompt-core`):
  - `bin/bridge/src/proxy/secret.rs` — mint/load/`proxy_init`/`for_profile`
  - `bin/bridge/src/proxy/server.rs:63` — proxy verifies against `proxy_init()`
  - `bin/bridge/src/proxy/dispatch/auth.rs::verify_loopback_secret` — 403 + log text
  - `bin/bridge/src/gui/hosts/handlers.rs:178,426,480` — host profile writes `api_key = for_profile()`
  - `bin/bridge/src/cli/doctor/auth.rs:141` — presence-only loopback check
