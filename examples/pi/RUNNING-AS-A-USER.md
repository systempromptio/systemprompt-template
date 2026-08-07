# Running Pi as a named systemprompt user

Pi talks to the gateway with a credential, and the gateway resolves that
credential to a user row. Every session, request, token count, cost, and
governance decision the run produces is stored against that user. So the
credential you install decides whose profile page fills up.

This guide sets Pi up to act as a user who already exists in your database
(you, a colleague, a demo account) rather than a synthetic one, and shows how
to drive it from the TUI, not just from scripts.

## Prerequisites

- The platform built and running: `just build`, then `just start`.
- Node.js, for the `pi` binary itself.
- `jq`, used by the setup scripts.

Note which port your profile uses. It is read from
`.systemprompt/profiles/<profile>/profile.yaml` (`api_server_url`), and the
scripts pick it up automatically. The examples below assume 8099; substitute
your own.

## One-time setup

```bash
examples/pi/setup.sh
```

Installs the `pi` binary if missing, writes the `systemprompt` provider into
`~/.pi/agent/models.json`, installs the governance extension into
`~/.pi/agent/extensions/`, and installs the branded theme.

Optional, and worth doing before a live demo:

```bash
examples/pi/routes.sh   # per-model gateway routes, so each model is separately governable
just start              # restart so the new routes register as authz entities
```

**Ordering matters.** `setup.sh` writes an admin session JWT into the
credential directory. Always run it *before* choosing your user, never after.
If you re-run it later, re-run the next step too, or you will quietly be
acting as the admin again.

## Choose the user

```bash
examples/pi/new-user.sh
```

With no arguments this lists the users already in the database, admins first,
and asks which one to act as:

```
  Select the user to act as:

     1) ed@systemprompt.io [admin]  ed@systemprompt.io  (user,admin)
     2) admin@localhost.dev [admin]  admin  (admin,user)
     3) pi-demo@demo.local  Pi Demo  (user)
     n) create a new demo user

  Choice [1]:
```

Pick a number to act as an existing user, or `n` to create a fresh demo
account. Skip the menu entirely by naming an email:

```bash
examples/pi/new-user.sh ed@systemprompt.io
```

The script then issues that user a personal access token, mints their
governance token, mints a gateway session, and smoke-tests `/v1/messages` as
them. It finishes by printing the dashboard links for whoever was selected.

**Roles are never modified for a user you did not create.** Minting a
governance token normally requires admin rights, which older versions obtained
by promoting the target user and demoting them afterwards. That would strip a
real admin of their role. The script now mints directly when the selected user
is already an admin, uses the promote-then-demote path only for accounts it
created itself, and otherwise refuses and prints the commands for you to run
deliberately.

## Use the TUI

```bash
pi
```

Then `/model` (Ctrl+L) and pick one of the gateway models under the
`systemprompt` provider: Claude Sonnet 4.6, Claude Opus 4.8, GPT-5 mini,
Gemini 2.5 Flash. Type normally. Every turn goes through the gateway as your
chosen user.

Nothing needs configuring between the previous step and this one. The provider
entry stores `"apiKey": "!cat ~/.config/systemprompt-pi/token"`, a shell
reference rather than a copied value, so Pi re-reads the credential file at
launch. The governance extension reads its own token and the gateway URL from
the same directory when a session starts.

Watch it land while you type: open `/admin/demo/trace` in a browser and select
your session. One Pi conversation is one session row, and the timeline shows
prompt gate verdicts, tool gate verdicts, model calls, and tool fires in the
order they happened.

## Switch users

```bash
examples/pi/new-user.sh    # pick someone else
# restart pi
```

No config edit. The `!cat` indirection means the next launch reads the new
token. Inside an already-running Pi, `/reload` refreshes the extension but not
the provider credential, so restart the binary when changing identity.

## What gets written

Everything lives in `~/.config/systemprompt-pi/`:

| File | Contents | Used by |
|------|----------|---------|
| `token` | `sp-live-…` personal access token | Pi's provider, for `/v1/messages` |
| `hook-token` | plugin JWT | the governance extension, for `/hooks/govern` |
| `base-url` | gateway origin for this profile | the extension, so a non-default port needs no edit |
| `session` | the session minted at setup time | `trace.sh` and the walkthrough |
| `user.json` | id, email, name, roles, is_admin | the demos, to name the caller and expect the right verdicts |

Two credentials because two endpoints disagree about what they accept.
`/v1/messages` takes the PAT. `/hooks/govern` validates a JWT and rejects a
PAT outright, so the governance extension carries its own.

## Who you pick changes what the demo proves

Admins are exempt from two of the four governance policies: `scope_check` and
`tool_blocklist` short-circuit on admin scope. `secret_scan` and `rate_limit`
have no exemption and apply to everyone.

So `demo/governance/09-pi-agent.sh` behaves differently by design:

| Case | Policy | Non-admin user | Admin user |
|------|--------|----------------|------------|
| benign prompt | none | allow | allow |
| AWS key in the prompt | `secret_scan` | deny | deny |
| `.env` containing a GitHub PAT | `secret_scan` | deny | deny |
| `delete_records` | `tool_blocklist` | deny | allow, exempt |
| `mcp__systemprompt__list_agents` | `scope_check` | deny | allow, exempt |
| read a source file | none | allow | allow |

The script detects which case applies and asserts the real outcome either way,
so it passes as an honest report rather than narrating a denial that did not
happen. Choose a non-admin when the audience needs to see tool calls blocked.
Choose yourself when the point is that your own usage is attributed correctly.

## Verify attribution

```bash
./demo/governance/09-pi-agent.sh
```

Free and deterministic. It replays the six wire events the extension sends,
asserts each verdict, and asserts that every resulting decision carries the
acting user's id. A run that attributed to nobody fails rather than printing a
link to an empty profile.

Then check the user's own pages:

```
http://localhost:8099/admin/user?id=<id>            profile, events, costs
http://localhost:8099/admin/models?user_id=<id>          per-user model access
http://localhost:8099/admin/demo/trace?session=<session> the governed timeline
```

Use `localhost` in a browser, not `127.0.0.1`. The dashboard session cookie is
set without a `Domain` attribute, which makes it host-only: a login at
`localhost` does not authenticate a page opened at `127.0.0.1`, and you get
bounced to the login screen. The scripts print browser links using the host
from your profile for this reason, while their own curl calls use `127.0.0.1`,
because `localhost` sometimes resolves to a `[::1]` the server is not bound to.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Usage appears under the admin, not your user | `setup.sh` was run after `new-user.sh` | re-run `new-user.sh` |
| Login page instead of the dashboard | opened a `127.0.0.1` link after logging in at `localhost` | use the `localhost` form |
| `no user '<email>' in the … database` | typo, or the wrong profile | the error lists the real users; pick one |
| Script refuses to mint a governance token | selected a pre-existing non-admin, whose roles it will not change | run the printed promote command, re-run, then demote |
| Tool calls are not being gated | extension not loaded | `/reload` in Pi, or check `~/.pi/agent/extensions/` |
| `Error: fetch failed` at startup, then `400 missing required x-session-id header` | the extension could not mint a session, so it sent no session header | update the extension (`cp examples/pi/extensions/governance.ts ~/.pi/agent/extensions/systemprompt-governance.ts`) and restart Pi; it now falls back to an already-issued session and warns instead |
| Connection refused from a script | `localhost` resolving to `[::1]` | `BASE_URL=http://127.0.0.1:8099 <script>` |

## See also

- [`README.md`](README.md) for how the integration works: provider mapping,
  the two credentials, the extension's event mapping.
- [`WALKTHROUGH.md`](WALKTHROUGH.md) for the full demo story, including
  per-user model governance from `/admin/models`.
