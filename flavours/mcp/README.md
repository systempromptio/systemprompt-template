# mcp

Hosted MCP servers and the marketplace on top of the gateway, governed per user. Clients reach the `systemprompt` MCP toolset and the plugins, skills and managed resources an administrator grants them, and every tool call passes the governance pipeline.

## What it serves

In: everything in `gateway`, plus hosted MCP servers (`/api/v1/mcp/registry`), the marketplace, plugins, skills, managed resources, hooks and the project pages.

Out: agents, agent runs, notifications, organisational memory, the public site, Slack. As in `gateway`, the console has no sessions, conversations, history or version-analysis pages.

`flavours/mcp/flavour.yaml` lists the exact routes, console pages, CLI
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
6. publishes the console assets; `/` sends the browser to `/admin`.

## Set it up locally

```bash
just setup mcp --anthropic-key <key> [--http-port 8080] [--pg-port 5432] [--admin-email you@example.com]
FLAVOUR=mcp just start
```

Setup builds `target/flavour-mcp/`, starts Postgres from
`.systemprompt/docker/mcp.yaml`, writes `.systemprompt/profiles/mcp/`,
copies the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/mcp/services` (with this directory's `services/`
overrides on top), then does the steps above. Without a key the CLI prompts for a provider; `--openai-key` and `--gemini-key` are accepted too. `--admin-email`
defaults to `git config user.email`; a bare re-run keeps the ports chosen at
first setup; `--no-build` uses binaries already in `target/flavour-mcp/`.
An edit under `services/` takes effect on the next `just setup mcp`.
`FLAVOUR=mcp just stop` stops it.

## Build and run the image

```bash
just docker-build mcp                         # systemprompt-template:mcp
ADMIN_EMAIL=you@example.com ANTHROPIC_API_KEY=<key> \
  FLAVOUR=mcp docker compose -p sp-mcp up -d
just smoke mcp --no-build                     # boot it and assert flavour.yaml's smoke
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
docker compose -p sp-mcp exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` link (valid for 15 minutes) on
the host `EXTERNAL_URL` names (default `http://localhost:8080`), register the
passkey, then sign in at `/admin/login` with the same e-mail.

## Known limits

- No agent or conversation pages, and no public site.
- AD FS single sign-on verifies an assertion but cannot issue a session on
  this core; the passkey is the only browser sign-in.
