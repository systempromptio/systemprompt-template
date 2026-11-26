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

## Status on core `future-compose` (aec7946bd)

The image builds, but a fresh install does not migrate: core's `ai` and
`analytics` schemas create views, foreign keys and triggers over tables the
`mcp-servers` and `agents` members own (`mcp_tool_executions`,
`mcp_artifacts`, `agent_tasks`, `task_messages`, `user_contexts`), and the
kernel `files` schema references `markdown_content` (`content`). First boot
stops at `infra db migrate` with `relation "mcp_tool_executions" does not
exist`. Until core installs each of those objects only with the member that
owns the referenced table, this flavour (like every subset flavour) cannot
start; `just smoke gateway --no-build` reports exactly that.

## Run it locally

```bash
just setup gateway --anthropic-key <key>    # or --openai-key / --gemini-key
FLAVOUR=gateway just start
```

Setup builds `target/flavour-gateway/release/systemprompt`, starts Postgres from
`.systemprompt/docker/gateway.yaml`, writes `.systemprompt/profiles/gateway/`, stages
the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/gateway/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin.

## Run the image

```bash
just docker-build gateway                    # systemprompt-template:gateway
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
  FLAVOUR=gateway docker compose -p sp-gateway up
just smoke gateway --no-build                # boot and assert the smoke expectations
just metrics gateway                         # target/flavour-metrics/gateway.json
```

The image uses the `slim` runtime (no demo/, no hey). Compose reads
`ADMIN_EMAIL` (required on first boot) and one of `ANTHROPIC_API_KEY`,
`OPENAI_API_KEY` or `GEMINI_API_KEY` from the shell or `.env`; it publishes
the app on `HTTP_PORT` (8080) and Postgres on `PG_PORT` (5432). First boot
waits for Postgres, runs `admin setup` against `DATABASE_URL`, migrates and
creates the bootstrap admin for `ADMIN_EMAIL`; later boots reuse the profile
in the `app_profiles` volume.

Measured on this core (image, 24 cores): binary 70.2 MB, image 258.5 MB
(92.3 MB gzip, 10 layers), 616 crates in the closure, clean image build 648 s.

## Dashboard

`/` serves a landing page that sends the browser to `/admin/` (the kernel
owns `/`, so this is the `index.html` the console publishes at boot).
`/admin/` serves the base console (overview, users, groups, roles and
devices, governance, access control, sync, usage analytics) and the gateway
pages (policies, routes, quotas, request and cost analytics). There is no public
site. No MCP, plugin, skill or agent pages.

The sidebar, the home cards and the landing page come from
`flavours/gateway/services/web/config/dashboard.yaml`, which overrides
`services/web/config/dashboard.yaml`. Naming an item whose page this
flavour does not compile refuses boot with the console feature it needs.

## CLI groups

`admin`, `analytics`, `build`, `cloud`, `core`, `infra`, `plugins`, `web`. The
marketplace and MCP subcommands and `admin agents` are not compiled in.

## Configuration

`profile.overlay.yaml` sets nothing. `services/content/config.yaml` (no
sources) and `services/skills/` (empty) exist only because core's config
loader requires both paths in every composition. `services/scheduler/config.yaml` lists
only the jobs this flavour compiles (core refuses to boot on a configured job
no member contributes), and `services/config/config.yaml` includes only the
files of the subtrees it ships.
