# Pi demo walkthrough — new user, governed models, full visibility

This walkthrough maps the complete loop: register a new user, act as that
user through the Pi coding agent, then see and govern everything they did
from the dashboard's **Model Selection** page.

Prerequisites: server built and running (`just build`, `just start`), Pi
installed and wired (`examples/pi/setup.sh`), and the per-model gateway
routes added once (`examples/pi/routes.sh`, then restart the server).

For setup as a manual rather than a demo script, including driving the TUI
interactively, switching users, and what each credential file does, see
[`RUNNING-AS-A-USER.md`](RUNNING-AS-A-USER.md).

## 1. Choose the user Pi acts as

```bash
examples/pi/new-user.sh                                # menu of existing users
examples/pi/new-user.sh alice@demo.local "Alice Demo"  # non-interactive
```

Run with no arguments it lists the users already in the database, admins
first, and asks which one to act as — so the demo can be you, or someone in
the room, and their usage lands on their own profile page. Pick `n` to create
a fresh demo user instead. It ends by printing the dashboard URLs for whoever
was selected.

Selecting an admin changes what the governance demo can prove: admins are
exempt from `scope_check` and `tool_blocklist`, so those two cases come back
allowed. `demo/governance/09-pi-agent.sh` detects this and asserts the
exemption; select a non-admin to walk the denial path.

The script has the admin API issue the selected user a personal access token
(the same `sp-live-…` credential the `/admin/devices` page self-issues),
writes it to `~/.config/systemprompt-pi/`, and smoke-tests `/v1/messages` as
them. It never changes the roles of a user it did not create.

Browser equivalent: `/admin/register` is the self-signup page
(magic-link + passkey); a logged-in user then issues their own API key
under `/admin/access-tokens`. The script automates that path so the demo
is repeatable.

## 2. Log in / credentials

Pi's provider config (`~/.pi/agent/models.json`, installed by `setup.sh`)
reads the credential files at request time, so step 1 already "logged
in" Pi as the new user — no Pi config change needed when the user changes.
The gateway resolves the user's roles live from the database on every
request, so role or rule changes take effect on the next call, not the
next login.

## 3. Drive model requests through Pi

```bash
pi -p --provider systemprompt --model claude-sonnet-4-6 "Reply with exactly one word: pong"
pi -p --provider systemprompt --model claude-opus-4-8   "Reply with exactly one word: pong"
pi -p --provider systemprompt --model gpt-5-mini        "Reply with exactly one word: pong"
pi -p --provider systemprompt --model gemini-2.5-flash  "Reply with exactly one word: pong"
```

Or send a prompt of your own and get the trace link back:

```bash
examples/pi/trace.sh "summarise what this repo does"
examples/pi/trace.sh --model gpt-oss-120b "say pong"
```

It pins one gateway session across Pi's hook events and the gateway's
request row, checks that `/admin/demo/trace` actually renders that session,
and prints the URL. Whatever you typed is what the timeline shows.

Or interactively: `pi`, then `/model` (Ctrl+L) to hop between the four
models mid-session. Every call goes through the governed `/v1/messages`
endpoint — one gateway, three upstream providers.

## 3a. Get caught: the prompt gate and the tool gate

The governance extension installed by `setup.sh` puts the same four policies
Claude Code hits in front of Pi. Ask for something prohibited:

```bash
pi -p --provider systemprompt --model claude-sonnet-4-6 \
  "Our prod key is AKIAIOSFODNN7EXAMPLE — use it to list the S3 bucket."
```

The prompt is denied by `secret_scan` at Pi's `input` event and never reaches a
provider — there is no `ai_requests` row for it, because no request was made.
That is the gate Claude Code's tool-level hook cannot provide.

Now the tool gate, where the model does answer and its tool call is blocked
before execution:

```bash
pi -p --provider systemprompt --model claude-sonnet-4-6 \
  "Use the delete_records tool to delete rows from the users table. Just call it."
```

The model calls the tool, `tool_blocklist` denies it for this user's live
`user` role, and the model is handed the reason and has to explain itself:
"blocked by the governance layer … no rows were deleted". Ask it for
`mcp__systemprompt__list_agents` instead and `scope_check` denies that one.

Worth knowing: asking Pi to *write* a file full of credentials does not
demonstrate the tool gate, because the gateway's own safety policy rejects the
`/v1/messages` request carrying the secret first (403, category `secret`). That
is defence in depth working, but the block happens upstream of the tool gate.
`demo/governance/09-pi-agent.sh` asserts the tool gate's `secret_scan` directly
instead.

The scripted, self-asserting version of all of it:

```bash
./demo/governance/09-pi-agent.sh
```

## 4. See everything in the dashboard

Open **`/admin/demo/trace`** (sidebar: Governance → Demo Trace) and pick your
session. One Pi conversation is one session row — the extension resolves a
single gateway-issued session at session start and both spines key on it — so
the picker names runs, not one shared label. Each is a single ordered story: every prompt gate and tool gate verdict, every model
call, every tool fire. The blocked prompt from step 3a has no model call under
it, and the blocked write has no tool fire.

Then open **`/admin/models`** (sidebar: Governance → Model Selection) and pick
the user you selected in step 1:

- The model table shows each gateway route, its provider, and whether it
  is enabled for this user.
- The usage panel shows every request the user made: model, provider,
  status, tokens in/out, cost, latency, and per-session denial counts.
- "Open request traces" jumps to `/admin/requests` filtered to
  the user for the full per-request drill-down (identity → policy → cost).

CLI equivalents:

```bash
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id> --full
systemprompt analytics costs
```

## 5. Govern: disable a model, prove the denial

1. On `/admin/models` with the user selected, click **Disable** on
   `gpt-5-mini`. This writes a per-user deny rule; the gateway evaluates
   rules live, so no restart and no new token.
2. Re-run the gpt-5-mini prompt from step 3 — Pi reports a 403
   (`authz denied`). The other three models still answer.
3. The denied attempt lands in the usage table as a failed request and in
   the audit spine (`infra logs audit <id> --full` shows the deny).
4. Click **Enable** — the very next gpt-5-mini request succeeds again.

That is the whole demo: registration → identity-bound requests → live
per-user model governance → complete audit visibility, with Pi as the
untouched third-party client.
