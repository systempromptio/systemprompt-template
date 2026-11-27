# full

Everything the template builds: the AI gateway, hosted MCP and the marketplace, agents, the public site, the whole admin console and Slack. Use it to evaluate the platform and to run the demos in `demo/`.

## What it serves

In: the union of `agents` and `site`, core's `reference` composition, issues and Slack; the image also carries `demo/` and `hey` (`runtime: full`).

Out: nothing the template builds.

`flavours/full/flavour.yaml` lists the exact routes, console pages, CLI
groups and jobs smoke asserts present and absent. The core members it links
and the rules every flavour follows are in
[docs/flavours/README.md](../../docs/flavours/README.md).

## What a fresh install does

On first start, local or in the image, the install:

1. waits for Postgres;
2. writes a profile with `admin setup` (a provider key is required) and
   mints `mcp_credential_broker_secret` into its secrets;
3. migrates the database;
4. creates the bootstrap admin for the admin e-mail;
5. applies the config planes;
6. runs the publish pipeline, so the public site is served at `/`.

## Set it up locally

```bash
just setup full --anthropic-key <key> [--http-port 8080] [--pg-port 5432] [--admin-email you@example.com]
FLAVOUR=full just start
```

Setup builds `target/flavour-full/`, starts Postgres from
`.systemprompt/docker/full.yaml`, writes `.systemprompt/profiles/full/`,
copies the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/full/services` (with this directory's `services/`
overrides on top), then does the steps above. Without a key the CLI prompts for a provider; `--openai-key` and `--gemini-key` are accepted too. `--admin-email`
defaults to `git config user.email`; a bare re-run keeps the ports chosen at
first setup; `--no-build` uses binaries already in `target/flavour-full/`.
An edit under `services/` takes effect on the next `just setup full`.
`FLAVOUR=full just stop` stops it.

## Build and run the image

```bash
just docker-build full                         # systemprompt-template:full
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
  FLAVOUR=full docker compose -p sp-full up -d
just smoke full --no-build                     # boot it and assert flavour.yaml's smoke
```

Compose reads `ADMIN_EMAIL` (required on first boot) and one of `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `GEMINI_API_KEY` from the
shell or `.env`, and publishes the app on `HTTP_PORT` (8080) and Postgres on
`PG_PORT` (5432). Later boots reuse the profile in the persisted volume. A
supplied profile (`SYSTEMPROMPT_PROFILE_DIR`) must carry
`mcp_credential_broker_secret`, or the server refuses to start.

## Sign in

The console signs in with a passkey. Issue a one-time link for the bootstrap
admin:

```bash
docker compose -p sp-full exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` link (valid for 15 minutes) on
the host `EXTERNAL_URL` names (default `http://localhost:8080`), register the
passkey, then sign in at `/admin/login` with the same e-mail.

## Known limits

- The largest image and dependency closure of the five; see [metrics](../../docs/flavours/metrics.md).
- AD FS single sign-on verifies an assertion but cannot issue a session on
  this core; the passkey is the only browser sign-in.
