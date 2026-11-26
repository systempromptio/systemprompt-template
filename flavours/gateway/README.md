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

The console signs in with a passkey. A fresh install's bootstrap admin has
none, so issue a one-time link for it:

```bash
docker compose -p sp-gateway exec app systemprompt admin users webauthn \
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

`/` serves a landing page that sends the browser to `/admin`. The kernel
owns `/`, so this is the `index.html` the console publishes at boot; there is
no public site.

The sidebar has the five sections of
`flavours/gateway/services/web/config/dashboard.yaml`, which overrides
`services/web/config/dashboard.yaml`:

- Gateway: overview, analytics, requests.
- Policy: gateway, policies, quotas, governance, approvals, secrets audit.
- People & access: users, groups, projects, roles, access control, devices.
- Platform: code sync, configuration, data lifecycle, observability.
- Account: profile, settings.

`/admin` lands on the overview with the six home cards (Spend, Requests,
Models, Policies, Quotas, Audit). There are no MCP, plugin, skill or agent
pages, and no sessions, conversations or history pages: those read the
conversation record, which only a build with agents installs. Naming an item
this flavour does not compile refuses boot. The smoke expectations
(`flavour.yaml`) list the pages a signed-in admin gets 200 from and the
conversation pages that answer 404; on this tree they have not been run yet,
because the image needs the core fix in Status.

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

The web extension installs only the schema files whose tables this build
links (`extensions/web/src/schemas.rs`): the gateway's request scopes, audit
trigger and request-log expiry, and nothing that reads the MCP, marketplace,
agents or content members' tables.

## Known core gaps seen here

- The `trace_facts_rollup` job warns on every tick:
  `function trace_facts_mcp(text[]) does not exist`. Core's
  `refresh_trace_facts` (`crates/infra/database/schema/trace_facts.sql:67-68`)
  calls the `mcp` and `agent` members' contributor functions
  unconditionally.
- Analytics report routes and commands that read a view owned by a member
  this flavour lacks answer a database error.
