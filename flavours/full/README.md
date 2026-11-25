# full

Everything the template builds: the AI gateway, hosted MCP servers and the
marketplace, agents, the public site and content pipeline, Slack, and the whole
admin console. It is the flavour the demos in `demo/` and the published image
use.

## Run it locally

```bash
just setup full --anthropic-key <key>    # or --openai-key / --gemini-key; no key prompts on a TTY
just start                               # FLAVOUR defaults to full
```

Setup builds `target/flavour-full/release/systemprompt`, starts Postgres from
`.systemprompt/docker/full.yaml`, writes `.systemprompt/profiles/full/`, links
every `services/` subtree into `.systemprompt/flavours/full/services`, migrates,
creates the bootstrap admin and publishes the site. `--http-port` and
`--pg-port` run a second install beside another; a re-run keeps the ports it
chose first.

## Run the image

```bash
just docker-build full                   # systemprompt-template:full
FLAVOUR=full docker compose -p sp-full up
just smoke full --no-build               # boot and assert the smoke expectations
```

The image uses the `full` runtime: the slim runtime plus `demo/` and `hey`. It
needs one provider key (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or
`GEMINI_API_KEY`) and `ADMIN_EMAIL` on first boot.

## Configuration

`profile.overlay.yaml` sets nothing: `full` runs the profile `admin setup`
writes. `flavour.yaml` lists what the smoke test expects: the site, sitemap,
JWKS and admin routes, the `admin`, `analytics`, `build`, `cloud`, `infra`
and `plugins` CLI groups, the `publish_pipeline` job, the `systemprompt` MCP
server and `/v1/models`.
