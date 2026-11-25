# site

The public site and its content pipeline, with no AI: pages, documentation
and blog from `services/content`, the homepage and skills page, the sitemap,
`robots.txt` and `llms.txt`, files with PDF export, identity and the base
console. Use it to publish a governed site without running models.

## What is in, what is out

In: content and the generator, the publish pipeline and its sub-jobs,
content analytics, files and PDF export, identity and SSO, usage analytics
and the base console with its content pages.

Out: AI providers and `/v1/models`, hosted MCP and the marketplace, agents,
agent runs, notifications, the brain, OpenTelemetry, the bridge surface,
cloud deploy, Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`.

## Run it locally

```bash
just setup site                             # no provider key
FLAVOUR=site just start
```

Setup builds `target/flavour-site/release/systemprompt`, starts Postgres from
`.systemprompt/docker/site.yaml`, writes `.systemprompt/profiles/site/`, stages
the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/site/services` (with this directory's `services/`
overrides on top), migrates and creates the bootstrap admin. Setup then publishes the site.

## Run the image

```bash
just docker-build site                    # systemprompt-template:site
FLAVOUR=site docker compose -p sp-site up
just smoke site --no-build                # boot and assert the smoke expectations
```

The image uses the `slim` runtime. It needs no provider key; `ADMIN_EMAIL` on first boot.

## Dashboard

`/` is the published site; `/admin/` serves the base console and the content
pages. No gateway, MCP or agent pages.

## CLI groups

`admin`, `analytics`, `core`, `infra`, `plugins`, `web`. `cloud` is not
compiled in.

## Configuration

`profile.overlay.yaml` sets nothing. `services/scheduler/config.yaml` lists
only the jobs this flavour compiles (core refuses to boot on a configured job
no member contributes), and `services/config/config.yaml` includes only the
files of the subtrees it ships.
