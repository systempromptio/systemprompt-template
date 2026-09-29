---
title: "Connect OpenCode"
description: "Route OpenCode through the gateway: one installer flag writes the provider block and the API key, so every prompt is governed and audited instead of being refused with 403."
author: "systemprompt.io"
slug: "connect-opencode"
keywords: "opencode, connect, install, bridge, openai compatible, provider, auth.json, loopback proxy, 403, no loopback credential presented, mcp"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "opencode"]
published_at: "2026-09-04"
updated_at: "2026-09-28"
after_reading_this:
  - "Connect OpenCode to the gateway with one installer flag"
  - "Know which files the bridge writes and which tier each lands in"
  - "Diagnose a 403 from the loopback proxy without guessing at credentials"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Connect OpenCode

[OpenCode](https://opencode.ai/) supports many model wires. The bridge registers
it against the gateway's OpenAI-compatible endpoint, as a provider called
`systemprompt` pointed at a local loopback proxy — so OpenCode never holds a
model vendor's key, and every request is governed, priced, and audited like any
other.

Use your administrator's gateway HTTPS URL and trusted manifest public key; the examples below use `https://gateway.example.com` as a placeholder. Install OpenCode using your organization's approved method before enrolment.

## 1. Get a connect code

Sign in and open your profile page, then click **Generate a connect code** to mint a one-shot
code. It is single-use and expires ten minutes after it is issued, so get the
installer command ready first and take the code last.

An administrator can mint one headlessly:

```bash
systemprompt admin bridge issue-code --user-id you@example.com
```

## 2. Install the bridge and enrol OpenCode

**Linux or WSL:** use the shell installer below. For macOS, follow [Install on macOS](/documentation/install-macos), then run the client-specific enrolment command. The shell installer is not a native Windows command.

```bash
GATEWAY_URL="https://gateway.example.com"
curl -fSLo install.sh "$GATEWAY_URL/files/downloads/install.sh"
less install.sh
sh install.sh --download-base "$GATEWAY_URL/files/downloads" \
  --gateway "$GATEWAY_URL" --code "YOUR_CONNECT_CODE" --host opencode
```

The OpenCode tab of your Profile page shows the same command with your code
filled in. The installer is published by your administrator; it downloads the
bridge, verifies its checksum, redeems the code for a durable token, and enrols
the clients you named. Read its output, then run `systemprompt-bridge doctor`.
If you were given a trusted manifest public key, pin it with
`systemprompt-bridge install --gateway "$GATEWAY_URL" --pubkey "..."` and run
`systemprompt-bridge sync` again.

`--host` is repeatable and accepts a comma-separated list, so a machine running
both clients takes `--host claude-code,opencode`. Left off entirely, the
installer enrols Claude Code and adds OpenCode only when the `opencode` binary
is **already** on your `PATH` — so if you install OpenCode later, re-run the
enrolment. The installer accepts `--host` only; `--hosts all` is a flag of
the bridge binary, below.

Already have the bridge? Enrol OpenCode on its own, without reinstalling:

```bash
systemprompt-bridge install --host opencode
```

`systemprompt-bridge install` also takes `--hosts all` to enrol every client this
build supports. Name either `--hosts all` or one or more `--host <id>`, never
both on the same line.

**Native Windows PowerShell:** complete the download, gateway trust, sign-in, and proxy-start steps in [Install on Windows](/documentation/install-windows), then run:

```powershell
& $Bridge sync
& $Bridge install --host opencode
& $Bridge doctor
opencode
```

Use the verified `$Bridge` executable path from that guide. Keep the app or proxy running. OpenCode enrolment is separate from Claude Code enrolment.

## 3. Verify

`systemprompt-bridge doctor` reports the proxy, the credential, and the synced
manifest in one pass, and is the fastest way to tell a local problem from a
gateway one.

The loopback proxy authenticates every request with a bearer token local to this
machine. Ask it for the model list:

```bash
curl -s -H "Authorization: Bearer $(cat ~/.config/systemprompt/bridge-loopback.key)" \
  http://127.0.0.1:48217/v1/models
```

A JSON list of model ids means the proxy is up, your gateway credential is
valid, and the wire OpenCode uses is answering.

The `curl` example above uses Linux default paths and port; use the actual endpoint reported by `systemprompt-bridge diagnostics` if they differ. Do not paste credentials or authentication-file contents into support logs.

On Linux, confirm OpenCode itself has the provider and the key:

```bash
grep -A12 '"systemprompt"' /etc/opencode/opencode.json ~/.config/opencode/opencode.json 2>/dev/null
grep -q systemprompt ~/.local/share/opencode/auth.json && echo "api key present"
```

On Windows, use `& $Bridge doctor`. After selecting a `systemprompt` model in OpenCode, send a short prompt and confirm its account, model, and timestamp in gateway [History](/admin/history). Verify a read-only MCP tool separately.

## 4. Use it

Run `opencode`, open the model picker, and choose a model under the
`systemprompt` provider. Enrolment writes a default, so the first prompt works
without picking anything.

## What the bridge writes

| Piece | Where | Tier |
|---|---|---|
| Provider block (`provider.systemprompt`) and the default `model` | `/etc/opencode/opencode.json` | admin-owned |
| Provider block, when `/etc` is not writable | `~/.config/opencode/opencode.json` | your own |
| API key for the `systemprompt` provider | `~/.local/share/opencode/auth.json` (`0600`) | your own |
| MCP connectors (`mcp.<name>`) | `~/.config/opencode/opencode.json` | your own |
| Managed skills | `~/.config/opencode/skills/` | your own |
| Skill-use reporting plugin (`systemprompt-hooks.js`) | `~/.config/opencode/plugin/` | your own |

The reporting plugin is written for the plugin that owns governance hooks
(`hooks.governance: true`) and removed when none does. It listens to
OpenCode's `chat.message` and `tool.execute.after` events and posts a
Claude-Code-shaped hook event to the loopback proxy's track route whenever the
built-in `skill` tool runs, naming the skill by its `plugin:skill` identity.
The bearer it carries is a per-plugin hook token derived by the bridge, never
the loopback secret or your API key; the proxy mints the gateway credential and
stamps the enrolled device, exactly as it does for Claude Code hooks. Those
events are what the admin console's Skill conversations and Resource
effectiveness pages count for OpenCode sessions.

### Session linking

The same plugin also registers a `chat.headers` hook. OpenCode identifies each
conversation by a `ses_…` id; the plugin derives a stable session id from it
and sends that id as an `x-opencode-session` header on every chat request
OpenCode makes to the loopback proxy. Its hook events carry the same id, plus
`x-systemprompt-host: opencode`. The proxy links each request to that session
before forwarding it, so a whole OpenCode conversation
— every request in it and the spend it incurs — is attributed to the skills it
invoked, the same way a Claude Code session is. Nothing in your OpenCode config
needs changing for this; the plugin is written by `systemprompt-bridge install
--host opencode` and refreshed by `sync`.

The device the proxy stamps on those requests is enrolled automatically when
you sign in to the bridge, and appears on the administrator's
[Devices](/admin/devices) page.

The provider block names `@ai-sdk/openai-compatible` as its `npm` package,
points `options.baseURL` at the loopback proxy, and carries an
`x-inference-protocol` header that tells the gateway which model families to
advertise to this client. OpenCode fetches that npm package on first use, so a
fully air-gapped machine needs it pre-seeded.

The bridge writes `opencode.json`. If you keep an `opencode.jsonc` instead, the
bridge reads it when probing but never writes to it, so the block will not
appear there.

On macOS the admin tier is `/Library/Application Support/opencode/` and on
Windows `%ProgramData%\opencode\`; the API key sits under the user's own
`.local/share/opencode/` and `%USERPROFILE%\.local\share\opencode\`. macOS also
honours an MDM tier above the managed file
(`/Library/Managed Preferences/ai.opencode.managed.plist`), which the bridge
reads but never writes.

Every foreign key in these files survives. The bridge owns the
`provider.systemprompt` object, the top-level `model` when it names our
provider, and the MCP entries whose URL points at the loopback proxy. It
rewrites those and nothing else, so a config you wrote by hand is preserved
across syncs.

### The admin tier is not enforcement

The provider block is written to the admin tier because OpenCode layers the
managed directory above user and project config, so it is the tier a personal
config will not casually override.

It is **not** governance. On Linux the file is mode `0644` with no lockdown and
no allowlist behind it, and where the bridge cannot write `/etc/opencode` it
falls back to your own config and says so. Nothing stops a determined user from
editing either tier or adding another provider. Governance is enforced at the
gateway — scope checks, secret scanning, blocklists, rate limits and the audit
trail all run there, on every request, regardless of what any local file says.

Note that `sudo systemprompt-bridge install --host opencode` does **not** work today:
the bridge resolves your credentials from `$HOME`, which under `sudo` is
`/root`, where no enrolment exists. Run the enrolment as your normal user.

## Choosing a model

Models are namespaced by provider, so they are written `systemprompt/<model>` —
for example `systemprompt/claude-sonnet-5`. Enrolment writes the list the
gateway currently offers this client and sets a default, so the picker shows
what your role is allowed. Ask for a model that is not on that list and the
gateway refuses it rather than silently substituting another.

The list is scoped to the model families the client advertises, so it is
narrower than the instance's full catalogue. To see everything the gateway can
serve you, call `/v1/models` without the protocol header:

```bash
curl -s -H "Authorization: Bearer <your-pat>" \
  https://gateway.example.com/v1/models
```

If a model you expect is missing from OpenCode's picker but present there, it is
a discovery filter rather than a permission problem — ask an administrator.

## Troubleshooting

| What you see | What it means | What to do |
|---|---|---|
| `403 forbidden: no loopback credential presented` | OpenCode reached the proxy with no `Authorization` header. Almost always the provider block or `auth.json` was never written: `sync` installs MCP connectors and no credential, so the connectors appear and inference does not. | `systemprompt-bridge install --host opencode`, then restart OpenCode. |
| `403 forbidden: bad loopback secret` | The credential is real but belongs to a different bridge install answering on this port — most often Windows and WSL2 sharing `127.0.0.1:48217`. The body names the config directory of the install that answered. | Decide which install owns the port, stop the other, then re-run the enrolment from the one you keep. |
| MCP servers are listed but every prompt fails | The same missing-credential case as the first row; the two are written by different steps. | As above. |
| `connection refused` on `127.0.0.1:48217` | The proxy is not running. | Keep the bridge app running, or run `systemprompt-bridge proxy` in a separate terminal (`& $Bridge proxy` in PowerShell). |
| The model list is empty, or a model is rejected | Your role does not reach that model, or the manifest has not synced yet. | `systemprompt-bridge sync`, then check the list with the `curl` above. |
| `--host opencode: this build does not offer the 'opencode' host` | The bridge binary you installed was built without the OpenCode integration. | Re-download the bridge from this instance rather than reusing an older binary. |
| Enrolment says the instance does not enable this host | OpenCode is not in the host list your instance publishes for you. | Ask an administrator to enable the `opencode` host, then `systemprompt-bridge sync` and re-run the enrolment. |
| The provider block landed in your own config, not `/etc` | `/etc/opencode` was not writable and Linux has no elevation path to offer. | It works as is. To use the admin tier, have an administrator create `/etc/opencode` writable by you, then re-run the enrolment. |
| Upstream errors naming a missing API key | The gateway has no credential configured for the provider behind that model. | Nothing to fix locally — report the model id to an administrator. |
| Skills you run in OpenCode do not appear on the Skill conversations page | The reporting plugin is missing (`~/.config/opencode/plugin/systemprompt-hooks.js`), the proxy is not running, or no plugin on your instance owns governance hooks. | `systemprompt-bridge sync`, confirm the file exists, keep the proxy running, then re-run the skill. |

## Removing it

Un-enrol OpenCode and leave everything else in place:

```bash
systemprompt-bridge uninstall --host opencode
```

In Windows PowerShell, use `& $Bridge uninstall --host opencode`.

This strips the bridge-owned provider block from both tiers, removes the
`systemprompt` entry from `auth.json` and deletes the reporting plugin,
leaving every other provider, plugin and key you added untouched.

To remove the bridge itself, `systemprompt-bridge uninstall` with no `--host`, or
`systemprompt-bridge uninstall --purge` to take the cached credentials and synced
manifest with it.

---

*Use a bridge build that supports OpenCode enrolment. A successful plugin sync alone does not configure inference.*
