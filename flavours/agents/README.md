# agents

Agents on top of the gateway and MCP hosting: A2A agents with contexts, tasks
and artifacts, headless agent runs (Claude Code, Codex CLI), notifications and
the brain (organisational memory). Use it for autonomous work without a public
site.

## What is in, what is out

In: everything in `mcp`, plus agents and the agent registry, agent runs,
notifications, the brain, the conversation judge and conversation rollups.

Out: the public site and the content pipeline, Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`.

## Run it locally

```bash
just setup agents --anthropic-key <key>     # or --openai-key / --gemini-key
FLAVOUR=agents just start
```

Setup builds `target/flavour-agents/release/systemprompt`, starts Postgres from
`.systemprompt/docker/agents.yaml`, writes `.systemprompt/profiles/agents/`, stages
the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/agents/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin.

## Run the image

```bash
just docker-build agents                    # systemprompt-template:agents
FLAVOUR=agents docker compose -p sp-agents up
just smoke agents --no-build                # boot and assert the smoke expectations
```

The image uses the `slim` runtime. It needs one provider key (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY`) and `ADMIN_EMAIL` on first boot.

## Dashboard

`/admin/` serves the base console, the gateway and MCP pages, and the agent
pages (sessions, conversations, contexts, traces, analysis, artifacts). `/`
redirects to the console.

The sidebar, the home cards and the landing page come from
`flavours/agents/services/web/config/dashboard.yaml`, which overrides
`services/web/config/dashboard.yaml`. Naming an item whose page this
flavour does not compile refuses boot with the console feature it needs.

## CLI groups

`admin` (with `admin agents`), `analytics`, `cloud`, `core`, `infra`,
`plugins`, `web`. `just kit-export` applies here too.

## Configuration

`profile.overlay.yaml` sets nothing. `services/scheduler/config.yaml` lists
only the jobs this flavour compiles (core refuses to boot on a configured job
no member contributes), and `services/config/config.yaml` includes only the
files of the subtrees it ships.
