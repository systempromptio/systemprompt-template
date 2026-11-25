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
FLAVOUR=gateway docker compose -p sp-gateway up
just smoke gateway --no-build                # boot and assert the smoke expectations
```

The image uses the `slim` runtime. It needs one provider key (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY`) and `ADMIN_EMAIL` on first boot.

## Dashboard

`/admin/` serves the base console (overview, users, groups, roles and
devices, governance, access control, sync, usage analytics) and the gateway
pages (policies, routes, quotas, request and cost analytics). `/` redirects to
the console: there is no public site. No MCP, plugin, skill or agent pages.

## CLI groups

`admin`, `analytics`, `cloud`, `core`, `infra`, `plugins`, `web`. The
marketplace and MCP subcommands and `admin agents` are not compiled in.

## Configuration

`profile.overlay.yaml` sets nothing. `services/scheduler/config.yaml` lists
only the jobs this flavour compiles (core refuses to boot on a configured job
no member contributes), and `services/config/config.yaml` includes only the
files of the subtrees it ships.
