# Astound Bridge

A branded, white-label build of the **systemprompt bridge** (credential helper +
plugin/MCP sync agent + local inference proxy) for Astound Digital. The desktop
app for Windows and macOS that connects Claude Desktop / Claude Code / Codex to
the Astound governance gateway.

This crate is intentionally tiny. All behaviour lives in the shared core library
(`systemprompt-core/bin/bridge`); here we only supply a `Brand` value and the
brand assets. The crate is a **standalone workspace** (its own `[workspace]`),
not a member of the main Astound server workspace, because it carries GUI
dependencies and ships on its own release cadence — exactly like core's bridge.

## Build & run

```bash
cd bridge
cargo build --release                 # host target
cargo run -- help                     # show Astound-branded help
cargo run -- gui                      # native settings UI (macOS/Windows only)
```

The GUI (winit + wry) compiles only on macOS/Windows; on Linux the crate builds
in headless/proxy mode.

Config, PAT, cache, and logs are isolated under the `astound` / `astound-bridge`
paths (e.g. `~/.config/astound/astound-bridge.toml`), and all env overrides use
the `ASTOUND_BRIDGE_` prefix (`ASTOUND_BRIDGE_PAT`, `ASTOUND_BRIDGE_CONFIG`, …).
The gateway URL is not overridable by env: it lives only in the config file
written by `login --gateway`, so a stale environment can never silently point
an authenticated bridge at a different gateway.

## Linux

There is no GUI on Linux (`gui` is macOS/Windows only). The headless inference
proxy takes its place: it listens on `127.0.0.1:48217`, swaps a loopback secret
for a fresh gateway JWT, and injects identity headers.

### Steady state: log in, run `claude`

An admin issues the user a one-shot enrolment code:

```bash
systemprompt admin bridge issue-code --user-id <uuid>
```

A signed-in user can also mint their own code from `/admin/profile`, which
prints the commands below with the code already filled in.

From a checkout of this repo, the code is the only argument either recipe needs:

```bash
just claude <code>    # Claude Code, connected, in a throwaway container
just connect <code>   # configure THIS host instead (writes ~/.profile etc.)
```

Without the repo, the installer does the same thing directly (it prompts for the
code if omitted):

```bash
curl -fsSL https://your-gateway/files/downloads/install.sh | sh -s -- \
  --download-base https://your-gateway/files/downloads --code <code>
```

The installer verifies the tarball checksum (refusing to proceed on a mismatch),
installs to `~/.local/bin` (or `/usr/local/bin` as root), installs Claude Code if
absent — that must precede `sync`, or the marketplace emitter skips silently —
redeems the code for a durable PAT via `login --code`, then runs
`install --apply --apply-schedule`, starts the proxy, syncs, and finishes on
`doctor`. Codes are short-lived and single-use; `--pat sp-live-…` is accepted
instead, and `--pubkey <base64>` pins the manifest signing key out of band
(without it the first sync trusts the key it is served, and says so).

`install --apply --apply-schedule` between them write:

| What | Where |
|---|---|
| `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN` | `~/.config/astound/env.sh` |
| a marker-delimited block sourcing that file | `~/.profile` |
| periodic plugin/MCP sync | `astound-bridge-sync.{service,timer}` (systemd user) |
| the loopback proxy, restarted on failure | `astound-bridge-proxy.service` (systemd user) |

The token is read from `~/.config/astound/bridge-loopback.key` when the file is
sourced, not baked in, so a rotated secret needs no rewrite. Re-running install
replaces the `~/.profile` block rather than appending a second one, and
`astound-bridge uninstall` removes both units, `env.sh`, and the block.

Where there is no systemd user bus (a container, WSL without systemd) the units
are still written and `--apply-schedule` warns instead of failing; run the proxy
by hand with `astound-bridge proxy`.

After a new login shell, `claude` works with no manual exports.

### Headless credentials

Device certificates are the supported unattended credential — they renew without
a browser, which a device-link grant cannot. Name the certificate in the config:

```toml
[mtls]
cert_keystore_ref = "~/.config/astound/device.pem"
```

`cert_keystore_ref` carries a *path* on Linux only. macOS addresses certificates
by Keychain label and Windows by cert-store thumbprint, and both ignore the
value — there, its presence just means "use mTLS".
`ASTOUND_BRIDGE_DEVICE_CERT` still works and takes precedence where both are set.

**Credential storage tiers down.** The OAuth client secret behind plugin hooks
prefers the freedesktop Secret Service, falls back to the kernel keyutils keyring
when no provider is present (headless servers, containers, CI), and finally to
this process's memory. Never a plaintext file — the secret is re-mintable, so
disk persistence would add an exfiltration target and buy nothing. Docker's
default seccomp profile denies `keyctl`/`add_key`, so keyutils is probed with a
real write/read round-trip before being accepted. `astound-bridge doctor` reports
which tier is in use — along with whether the proxy is listening and whether its
systemd unit is active.

**Runtime dependencies.** The binary dynamically links `libdbus-1`, `libsystemd`,
`libcap`, and `libgcrypt` — dbus because of the Secret Service store above. On a
minimal host these must be installed or the binary fails at exec with
`error while loading shared libraries`, before any of our diagnostics run:

```bash
sudo apt-get install -y libdbus-1-3 libcap2 libgcrypt20 libsystemd0   # Debian/Ubuntu
```

### Release tarball

```bash
just bridge-package-linux     # → dist/astound-bridge-linux-<arch>.tar.gz + .sha256
```

The recipe also publishes the tarball, its `.sha256`, and
`scripts/install-bridge.sh` (as `install.sh`) into `storage/files/downloads/`,
which the admin Bridge Setup page serves same-origin. The archive carries the
binary plus an `INSTALL.md` stating the above. Asset
names are load-bearing — they must match `DOWNLOAD_BASE_URL` in
`extensions/web/admin/src/handlers/ssr/ssr_bridge_setup.rs`, the links in
`storage/files/admin/templates/bridge-setup.hbs`, and `ARTIFACTS` in
`storage/files/js/pages/admin-bridge-setup.js`. Test the result on a machine
with no config using `just clean-client` (see `deploy/clean-client/`).

## macOS .app bundle

```bash
cargo build --release --target aarch64-apple-darwin
scripts/make-mac-app.sh --target aarch64-apple-darwin   # → AstoundBridge.app
```

## Icons

`assets/icon.svg` is the master Astound "A" mark (white on a near-black rounded
square, matching `storage/files/images/favicon-*`). The raster icons consumed by
the build are generated from it by `scripts/render-icons.py` (cairosvg + Pillow):

```bash
python3 scripts/render-icons.py
```

This regenerates, idempotently:

- `assets/window-icon-1024.png` — GUI window icon + macOS `.icns` source.
- `assets/tray-icon.png` — 44×44 tray icon (A on the rounded dark square, legible
  on both the dark macOS menu bar and a light Windows tray).
- `assets/app-icon.ico` — multi-resolution (16/32/48/256), embedded into the
  Windows `.exe` by `build.rs`. Rebuild (`cargo build --release`) after changing
  the icon so the new `.ico` is re-embedded.

Edit `assets/icon.svg` and rerun the script to change the mark. `assets/logo.svg`
is the full Astound wordmark, used by the GUI chrome.

> ⚠️ Still pre-release: set `default_gateway_url` in `src/main.rs` to the real Astound gateway host
(currently a `https://gateway.astounddigital.com` placeholder).

## Recipe: a new client bridge

The core/extension boundary makes a new white-label bridge a copy-and-swap job —
no forking of the bridge source:

1. Copy this `bridge/` crate to the new repo.
2. Replace everything in `assets/` with the client's marks + `theme.css`
   (override the `--sp-*` tokens; see `assets/theme.css`).
3. Edit the `Brand` const in `src/main.rs`: name, binary name, vendor, on-disk
   dir names, `env_prefix`, `default_gateway_url`, and chrome strings. Plugin
   ids are carried per-plugin in the gateway's signed manifest — there is no
   brand-level plugin-name field to set.
4. Update `build.rs` (Windows metadata), `macos/Info.plist` (bundle id + names),
   and `scripts/make-mac-app.sh` (bundle/app name).
5. Build and sign the per-platform artifacts locally — this repo runs no
   hosted CI, so `scripts/make-mac-app.sh`, `just bridge-package-linux`, and
   `just bridge-package-windows` are the release process. The Windows exe must
   be built for `x86_64-pc-windows-msvc` (the packaging recipe does this via
   cargo-xwin): only the msvc target statically links WebView2Loader, so a
   `-gnu` build ships a bare exe that dies at startup with
   "WebView2Loader.dll was not found".

Everything else — auth, sync, proxy, GUI, host integrations — is inherited from
core and stays in lockstep across all brands.
