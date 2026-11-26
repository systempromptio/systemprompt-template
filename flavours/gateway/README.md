# gateway

An AI gateway and nothing else: clients you already run (Claude Code, Codex,
any OpenAI- or Anthropic-compatible SDK) point at `/v1/messages`,
`/v1/chat/completions` or `/v1/responses`, and every request is authenticated,
routed to a provider, held to gateway policies and quotas, scanned, costed and
audited. Use it when model traffic is the only thing to govern.

## What is in, what is out

In: provider routing and `/v1/models`, gateway policies, quotas and the
month-window job, safety scanners, route entitlements, the bridge device
surface, OpenTelemetry ingest, cloud deploy, identity and SSO, usage analytics
and the base console with the console's gateway pages.

Out: hosted MCP servers and the marketplace (plugins, skills, managed
resources), agents, agent runs, notifications, organisational memory, the
public site and the content pipeline, Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`.

## Status

On core `future` snapshot b41c8b1f8 a fresh install migrates and the schema is
clean (`infra db doctor` reports nothing), but the server refuses to start:
`McpServer processes are not supervised on this host`. The `all` node role
always runs MCP reconciliation, and with no `mcp` member admitted there is no
process set to reconcile (core
`crates/entry/api/src/services/server/lifecycle/reconciliation.rs:48-50`).
Core needs to skip that phase when no MCP server is enabled, the way agent
reconciliation already does. Everything below was observed on an image built
from that snapshot with that one change.

## Run the image

```bash
just docker-build gateway                    # systemprompt-template:gateway
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
  FLAVOUR=gateway docker compose -p sp-gateway up -d
just smoke gateway --no-build                # boot and assert the smoke expectations
just metrics gateway                         # target/flavour-metrics/gateway.json
```

Compose reads `ADMIN_EMAIL` (required on first boot) and one of
`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY` from the shell or
`.env`. It publishes the app on `HTTP_PORT` (8080) and Postgres on `PG_PORT`
(5432). On first boot the entrypoint:

1. waits for Postgres;
2. runs `admin setup` against `DATABASE_URL`;
3. migrates;
4. creates the bootstrap admin for `ADMIN_EMAIL`;
5. applies the config planes.

Later boots reuse the profile in the `app_profiles` volume. A second
`infra db migrate` is a no-op (3.1 s, "No pending migrations"),
and a `docker compose restart app` is healthy again in about 8.4 s.

The image uses the `slim` runtime (no demo/, no hey).

Measured (target/flavour-metrics/gateway.json; 24 cores, 23 GB): binary 70.1 MB, image
206.4 MB (78.6 MB gzip, 11 layers), 616 crates in the closure,
clean image build 568 s. Fresh migrate 5.02 s, first boot to healthy
11.93 s, second boot 8.21 s, idle RSS 24 MiB.

## First login

The console signs in with passkeys. The bootstrap admin has none on a fresh
install, so link one first:

```bash
docker compose -p sp-gateway exec app \
  systemprompt admin users webauthn generate-setup-token --email you@example.com
```

Open the printed `http://localhost:8080/auth/link-passkey?token=…` link
(valid for 15 minutes) and create the passkey. Then sign in at
`http://localhost:8080/admin/login` with the same e-mail. `smoke` signs in
the same way, with a software authenticator
(`scripts/lib/console-login.py`).

`systemprompt admin session login` (a CLI session) fails in this flavour:
`no admitted member provides DynContextDirectory`. CLI sessions need the
`agents` member's context store (core
`crates/entry/cli/src/session/contexts.rs:62-67`). Commands that need no
session, such as `admin users`, `infra db` and `plugins capabilities`, work.

## Dashboard

`/` serves a landing page that sends the browser to `/admin`. The kernel
owns `/`, so this is the `index.html` the console publishes at boot; there is
no public site.

The sidebar has the five sections of
`flavours/gateway/services/web/config/dashboard.yaml`, which overrides
`services/web/config/dashboard.yaml`:

- Gateway: overview, analytics, requests, sessions, conversations.
- Policy: gateway, policies, quotas, governance, approvals, secrets audit.
- People & access: users, groups, projects, roles, access control, devices.
- Platform: code sync, configuration, data lifecycle, observability.
- Account: profile, my conversations, settings.

`/admin` lands on the overview with the six home cards (Spend, Requests,
Models, Policies, Quotas, Audit). Every page in that list answers 200 when
signed in. There are no MCP, plugin, skill or agent pages, and naming an item
this flavour does not compile refuses boot.

## CLI groups

Top level: `admin`, `analytics`, `build`, `cloud`, `core`, `infra`,
`plugins`, `web`. Under `admin`: `config`, `setup`, `bootstrap`,
`session`, `access-control`, `identity`, `licence`, `mcp`, `agent-host`,
`bridge`, `keys`, `users`. There is no `admin agents`.

## Configuration

`profile.overlay.yaml` sets nothing. `services/content/config.yaml` (no
sources) and `services/skills/` (empty) exist only because core's config
loader requires both paths in every composition.

`services/scheduler/config.yaml` lists only the jobs this flavour compiles:
core refuses to boot on a configured job that no member contributes.
`services/config/config.yaml` includes only the files of the subtrees it
ships.

The web extension installs the tool ledger, named contexts and the
marketplace tables as empty planes (`extensions/web/schema/20_member_planes.sql`),
so the conversation record, the last-seen roster and retention read the same
shape here as in `full`.

## Known core gaps seen here

- The `trace_facts_rollup` job warns on every tick:
  `function trace_facts_mcp(text[]) does not exist`. Core's
  `refresh_trace_facts` (`crates/infra/database/schema/trace_facts.sql:67-68`)
  calls the `mcp` and `agent` members' contributor functions
  unconditionally.
- Analytics report routes and commands that read a view owned by a member
  this flavour lacks answer a database error.
