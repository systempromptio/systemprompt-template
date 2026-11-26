# full

Everything the template builds: the AI gateway, hosted MCP servers and the
marketplace, agents and agent runs, notifications and organisational memory,
the public site and content pipeline, Slack, and the whole admin console. It is
the flavour the demos in `demo/` and the published image use.

## Run the image

```bash
just docker-build full                   # systemprompt-template:full
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
  FLAVOUR=full docker compose -p sp-full up -d
just smoke full --no-build               # boot a throwaway stack and assert flavour.yaml's smoke block
just metrics full --image                # target/flavour-metrics/full.json
```

Compose reads `ADMIN_EMAIL` (required on first boot: it names the bootstrap
admin) and one of `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY`
from the shell or `.env`; `EXTERNAL_URL` sets the public URL. It publishes
the app on `HTTP_PORT` (8080) and Postgres on `PG_PORT` (5432).

Every boot, in order: wait for Postgres; on the first boot only, `admin setup`
against `DATABASE_URL` (the profile lands in the `app_profiles` volume);
the profile overlays (`flavours/profile.base.yaml`, then this directory's
`profile.overlay.yaml`); `infra db migrate` (a no-op once settled);
`admin bootstrap`; `admin config apply` (the declared governance, access and
gateway planes; until they are applied the server reports itself degraded);
`infra jobs run publish_pipeline` (site pages, sitemap, robots.txt, llms.txt,
console CSS and assets); then the server, the `systemprompt` MCP server and
the `developer_agent` and `associate_agent` agents. First boot to healthy is
about 16 to 24 s, a restart about 11 to 17 s.

The image uses the `full` runtime: the slim runtime plus `demo/` and `hey`.

## First login

The console signs in with a passkey. A fresh install's bootstrap admin has
none, so issue a one-time link for it:

```bash
docker compose -p sp-full exec app systemprompt admin users webauthn \
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

`/admin` lands on the overview. The sidebar and home cards come from
`services/web/config/dashboard.yaml` (this flavour has no override): AI
activity, People & access, Analysis, Governance, Platform (marketplaces,
plugins, skills, MCP servers, gateway, sync, configuration, lifecycle,
observability), Account and Developer. Every item is compiled into `full`,
so the file is admitted at boot; an item a build lacks would refuse boot.

`/` is the public site, with `/blog`, `/documentation`, `/sitemap.xml`,
`/robots.txt` and `/llms.txt` published from `services/content`.

## CLI groups

Top level: `core`, `infra`, `admin`, `cloud`, `analytics`, `web`, `plugins`,
`build`. Under `admin`: `config`, `setup`, `bootstrap`, `session`,
`access-control`, `identity`, `licence`, `mcp`, `agent-host`, `agents`,
`bridge`, `issues`, `keys`, `users`.

## Run it locally

```bash
just setup full --anthropic-key <key>    # or --openai-key / --gemini-key; no key prompts on a TTY
just start                               # FLAVOUR defaults to full
```

Setup builds `target/flavour-full/release/systemprompt`, starts Postgres from
`.systemprompt/docker/full.yaml`, writes `.systemprompt/profiles/full/`,
applies `flavours/profile.base.yaml` and `profile.overlay.yaml`, stages every
`services/` subtree into `.systemprompt/flavours/full/services`, migrates,
creates the bootstrap admin and publishes the site. `--http-port` and
`--pg-port` run a second install beside another.

## Configuration

`profile.overlay.yaml` sets nothing of its own; the shared
`flavours/profile.base.yaml` grants the web member the secrets it declares
(`github_mcp_client_id`, `github_mcp_client_secret`, `web_connector_*`),
without which core refuses the composition. `flavour.yaml`'s smoke block
asserts the public site and sitemap, JWKS, the console login page, its CSS and
script, every CLI group above, the publish and content-analytics jobs, the
content, agent and MCP tables, the `systemprompt` MCP server's tools and
`/v1/models`.

## Measured

Image build on 24 cores: binary 115.8 MB, image 266.2 MB (102.8 MB gzip,
13 layers), 724 crates in the closure, clean image build 881 s, fresh-database
migrate 5 to 9 s, idle RSS about 96 MB. Times vary with host load.
