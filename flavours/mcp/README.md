# mcp

Hosted MCP servers and the marketplace on top of the gateway. Users reach the
template's `systemprompt` MCP server and any server a plugin declares, each
tool call governed and recorded per user; the marketplace distributes plugins
and skills to Claude Code and Claude Desktop through the bridge.

## What is in, what is out

In: everything in `gateway`, plus the MCP registry and hosting, the
`systemprompt` toolset, the marketplace, plugins, skills, managed resources,
hooks and connectors.

Out: agents, agent runs, notifications, organisational memory, the public site
and the content pipeline, Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`.

## Run the image

```bash
just docker-build mcp                    # systemprompt-template:mcp
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> FLAVOUR=mcp \
    docker compose -p sp-mcp up -d
just smoke mcp --no-build                # boot and assert the smoke expectations
```

The image uses the `slim` runtime. Compose passes `ADMIN_EMAIL`,
`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY` and
`EXTERNAL_URL` through to the app; at least one provider key is required.
The first boot waits for Postgres, writes the `docker` profile with
`admin setup`, installs the schema on the empty database (13 modules, 255
tables), creates the bootstrap admin from `ADMIN_EMAIL`, applies the declared
configuration, then starts the server and the `systemprompt` MCP server.
Measured: first boot to healthy about 17 s (6.8 s of it the fresh-database
migrate), a restart about 14 s; `infra db migrate` on a settled database is
a no-op of about 3 s, and `infra db doctor` is clean.

## First login

The console signs in with a passkey. A fresh install's bootstrap admin has
none, so issue a one-time link for it:

```bash
docker compose -p sp-mcp exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` link (valid for 15 minutes)
on the host `EXTERNAL_URL` names (default `http://localhost:8080`; the
passkey is bound to that host), register the passkey, then sign in at
`/admin/login` with the same e-mail. `scripts/flavour-smoke.sh` signs in
the same way, with a software authenticator (`scripts/lib/console-login.py`).
There is no other browser sign-in: the developer login link is removed, and
single sign-on (AD FS) verifies an assertion but cannot issue the session on
this core (see the flavours guide).

CLI commands that open a session (`admin session login`, `plugins mcp tools`)
fail on this core with "no admitted member provides DynContextDirectory": core
gives every CLI session a conversation context, which only the agents member
provides. Commands that need no session (`admin users …`, `infra db …`) work.

## Dashboard

`/` sends the browser to `/admin`, which redirects to `/admin/login`
until you are signed in and then lands on the overview. The sidebar, the home
cards and the landing come from `flavours/mcp/services/web/config/dashboard.yaml`,
which overrides `services/web/config/dashboard.yaml`:

- Activity: overview, analytics, requests
- MCP: MCP servers (`/admin/mcp`), plugins, skills, marketplaces
- Governance: governance, approvals, secrets, gateway, gateway policies, quotas
- People & access: users, groups, projects, roles, access control, devices
- Platform: sync, configuration, lifecycle, observability
- Account: profile, settings, connect, connectors

Naming an item whose page this flavour does not compile refuses boot with the
console feature it needs. Signed in, smoke gets 200 from /admin, analytics,
requests, mcp, plugins, skills, marketplaces, users, governance,
configuration and system/observability, and 404 from conversations, history,
sessions and analysis/versions: those read the conversation record, which only
a build with agents installs, so they are not compiled here. No agent,
conversation-judge or trace pages.

## CLI groups

Top level: `core`, `infra`, `admin`, `cloud`, `analytics`, `web`,
`plugins`, `build`. Under `admin`: `config`, `setup`, `bootstrap`,
`session`, `access-control`, `identity`, `licence`, `mcp`,
`agent-host`, `bridge`, `keys`, `users`. `admin agents` is not compiled
in. `just kit-export <marketplace> <dir>` (the `systemprompt-kit-export`
developer binary) applies to this flavour; it is built on demand, not shipped
in the image.

## Known limits

- Commands that open a CLI session (`admin session login`,
  `plugins mcp tools`) fail with "no admitted member provides
  DynContextDirectory": core issues every CLI session a conversation context,
  and only the agents member provides one. Commands that read the database
  directly (`admin users …`, `infra db …`) work.
- A signed-in `GET /admin/` (trailing slash) answers 404; use `/admin`.
- `/api/v1/agents` still answers with the A2A discovery document; the agent
  registry under it answers 404 only to an authenticated caller.

## Run it locally

```bash
just setup mcp --anthropic-key <key>        # or --openai-key / --gemini-key
FLAVOUR=mcp just start
```

Setup builds `target/flavour-mcp/release/systemprompt`, starts Postgres from
`.systemprompt/docker/mcp.yaml`, writes `.systemprompt/profiles/mcp/`, stages
the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/mcp/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin. This path was not
re-run for this README; the image path above was.

## Configuration

`profile.overlay.yaml` sets nothing. The `services/` overrides:

- `scheduler/config.yaml` lists only the jobs this flavour compiles (core
  refuses to boot on a configured job no member contributes);
- `config/config.yaml` includes only the files of the subtrees it ships;
- `content/config.yaml` is an empty content catalog, because core's config
  loader requires the file in every composition;
- `plugins/systemprompt/config.yaml` and
  `marketplaces/enterprise-demo/config.yaml` bundle no agents, since the
  services validator refuses a reference to an agent the build does not define.
