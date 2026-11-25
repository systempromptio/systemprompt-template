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

## Run it locally

```bash
just setup mcp --anthropic-key <key>        # or --openai-key / --gemini-key
FLAVOUR=mcp just start
```

Setup builds `target/flavour-mcp/release/systemprompt`, starts Postgres from
`.systemprompt/docker/mcp.yaml`, writes `.systemprompt/profiles/mcp/`, stages
the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/mcp/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin.

## Run the image

```bash
just docker-build mcp                    # systemprompt-template:mcp
FLAVOUR=mcp docker compose -p sp-mcp up
just smoke mcp --no-build                # boot and assert the smoke expectations
```

The image uses the `slim` runtime. It needs one provider key (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY`) and `ADMIN_EMAIL` on first boot.

## Dashboard

`/admin/` serves the base console, the gateway pages and the MCP pages (MCP
servers, plugins, skills, marketplace, connectors, bridge distribution). `/`
redirects to the console. No agent, conversation or trace pages.

## CLI groups

`admin`, `analytics`, `cloud`, `core`, `infra`, `plugins`, `web`, with the MCP
and marketplace subcommands. `admin agents` is not compiled in.
`just kit-export <marketplace> <dir>` (the `systemprompt-kit-export` developer
binary) applies to this flavour; it is built on demand, not shipped in the
image.

## Configuration

`profile.overlay.yaml` sets nothing. `services/scheduler/config.yaml` lists
only the jobs this flavour compiles (core refuses to boot on a configured job
no member contributes), and `services/config/config.yaml` includes only the
files of the subtrees it ships.
