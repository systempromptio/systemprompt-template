# agents

Agents on top of the gateway and MCP hosting: A2A agents with contexts, tasks
and artifacts, headless agent runs, notifications and the brain
(organisational memory). Use it for autonomous work without a public site.

## What is in, what is out

In: everything in `mcp` (the gateway, `/v1/models`, hosted MCP servers, the
marketplace), plus agents and the agent registry, agent runs, notifications,
the brain, the conversation judge and conversation rollups. A fresh install
migrates these members: database, events, users, logging, authz, oauth, mcp,
ai, managed_resources, agent, analytics, files, brain, notifications,
agent_runs, scheduler and web.

Out: the public site and the content pipeline (`/blog`, `/documentation`,
`/sitemap.xml`, `/robots.txt`, `/llms.txt`, `/api/public/links` and
`/api/public/search` answer 404), the `admin issues` CLI group, and Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`.

## Run it locally

```bash
just setup agents --anthropic-key <key>     # or --openai-key / --gemini-key
FLAVOUR=agents just start
```

Setup builds `target/flavour-agents/release/systemprompt`, starts Postgres from
`.systemprompt/docker/agents.yaml`, writes `.systemprompt/profiles/agents/`,
stages the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/agents/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin.

## Run the image

```bash
just docker-build agents                    # systemprompt-template:agents
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
    FLAVOUR=agents docker compose -p sp-agents up -d
just smoke agents --no-build                # boot and assert the smoke expectations
```

The compose file needs `ADMIN_EMAIL` and one provider key
(`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY`), from the shell
or `.env`. It publishes the app on `HTTP_PORT` (default 8080) and Postgres
on `PG_PORT` (default 5432); set `PG_PORT` when a local Postgres already
holds 5432. The image uses the `slim` runtime.

On first boot the entrypoint waits for Postgres, runs `admin setup` against
the compose database, migrates, bootstraps the admin user (`ADMIN_EMAIL`)
and applies the declared configuration. Every later boot waits, migrates (a
no-op on a settled database), verifies the admin and applies the
configuration again; the profile and secrets persist in the `app_profiles`
volume.

## First login

The console signs in with a passkey. A fresh install's bootstrap admin has
none, so issue a one-time link for it:

```bash
docker compose -p sp-agents exec app systemprompt admin users webauthn \
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

For the CLI inside the container, `admin session login` opens a CLI session
(observed in smoke: `plugins mcp tools` lists the `systemprompt` server's tools).

## Dashboard

`/admin` lands on the overview. The sidebar, the home cards and the landing
come from `flavours/agents/services/web/config/dashboard.yaml`, which
overrides `services/web/config/dashboard.yaml`:

- home cards: Agents (by person), Runs (traces), Conversations, Tools, MCP
  servers;
- sidebar: Agents (overview, analytics, by person, conversations, sessions,
  requests), Analysis, Runs (traces, tools), Platform (MCP servers, plugins,
  skills, marketplaces, sync, configuration, lifecycle, observability),
  Governance (governance, approvals, secrets, gateway, gateway policies,
  quotas), People & access, Account.

Naming an item whose page this flavour does not compile refuses boot with the
console feature it needs.

## CLI groups

Top level: `admin`, `analytics`, `build`, `cloud`, `core`, `infra`,
`plugins`, `web`. Under `admin`: `agents`, `agent-host`, `mcp`, `bridge`
among the rest; there is no `admin issues`. `plugins mcp tools -s systemprompt`
lists the hosted MCP toolset. `just kit-export` applies here too.

## Configuration

`profile.overlay.yaml` sets nothing. The overrides in `services/`:

- `scheduler/config.yaml` lists only the jobs this flavour compiles (core
  refuses to boot on a configured job no member contributes);
- `config/config.yaml` includes only the files of the subtrees it ships;
- `content/config.yaml` declares no content sources: core's config loader
  requires the file in every composition, and this flavour links no content.

## Measured

`bash scripts/flavour-metrics.sh agents --image` on 24 cores, with other
image builds running: binary 93.7 MB, image 226.2 MB (83.4 MB gzip, 11
layers), 631 crates in the closure, clean image build 682 s, fresh-database
migrate 9 to 10 s, boot to healthy 23 to 26 s (second boot 16 to 21 s), idle
RSS 84 to 99 MB. Times vary with host load.
