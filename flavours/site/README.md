# site

The public site and its content pipeline, with no AI: pages, blog and documentation from `services/content`, the sitemap, `robots.txt` and `llms.txt`, files with PDF export, and the base console.

## What it serves

In: content and the generator, the publish pipeline, content analytics, files and PDF export, identity, and the console's overview, users, governance, configuration and observability pages.

Out: AI providers and `/v1/models` (the composition has no gateway member), hosted MCP and the marketplace, agents, agent runs, notifications, organisational memory, OpenTelemetry, the bridge surface, the `cloud` CLI group, Slack. The console has no requests or analytics pages, no group or project management pages and no conversation pages.

`flavours/site/flavour.yaml` lists the exact routes, console pages, CLI
groups and jobs smoke asserts present and absent. The core members it links
and the rules every flavour follows are in
[docs/flavours/README.md](../../docs/flavours/README.md).

## What a fresh install does

On first start, local or in the image, the install:

1. waits for Postgres;
2. writes a profile with `admin setup` (no provider key is asked for) and
   mints `mcp_credential_broker_secret` into its secrets;
3. migrates the database;
4. creates the bootstrap admin for the admin e-mail;
5. applies the config planes;
6. runs the publish pipeline, so the public site is served at `/`.

## Set it up locally

```bash
just setup site [--http-port 8080] [--pg-port 5432] [--admin-email you@example.com]
FLAVOUR=site just start
```

Setup builds `target/flavour-site/`, starts Postgres from
`.systemprompt/docker/site.yaml`, writes `.systemprompt/profiles/site/`,
copies the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/site/services` (with this directory's `services/`
overrides on top), then does the steps above. `--admin-email`
defaults to `git config user.email`; a bare re-run keeps the ports chosen at
first setup; `--no-build` uses binaries already in `target/flavour-site/`.
An edit under `services/` takes effect on the next `just setup site`.
`FLAVOUR=site just stop` stops it.

## Build and run the image

```bash
just docker-build site                         # systemprompt-template:site
ADMIN_EMAIL=you@example.com \
  FLAVOUR=site docker compose -p sp-site up -d
just smoke site --no-build                     # boot it and assert flavour.yaml's smoke
```

Compose reads `ADMIN_EMAIL` (required on first boot) from the
shell or `.env`, and publishes the app on `HTTP_PORT` (8080) and Postgres on
`PG_PORT` (5432). Later boots reuse the profile in the persisted volume. A
supplied profile (`SYSTEMPROMPT_PROFILE_DIR`) must carry
`mcp_credential_broker_secret`, or the server refuses to start.

## Sign in

The console signs in with a passkey. Issue a one-time link for the bootstrap
admin:

```bash
docker compose -p sp-site exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` link (valid for 15 minutes) on
the host `EXTERNAL_URL` names (default `http://localhost:8080`), register the
passkey, then sign in at `/admin/login` with the same e-mail.

## Known limits

- No AI of any kind: no provider key, no gateway, no `/v1/models`.
- No group or project management in the console.
- AD FS single sign-on verifies an assertion but cannot issue a session on
  this core; the passkey is the only browser sign-in.
