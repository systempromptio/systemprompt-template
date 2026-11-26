# site

The public site and its content pipeline, with no AI: pages, documentation
and blog from `services/content`, the homepage, the sitemap, `robots.txt` and
`llms.txt`, files with PDF export, identity and the base console. Use it to
publish a governed site without running models.

## What is in, what is out

In: content and the generator, the publish pipeline and its sub-jobs,
content analytics, files and PDF export, identity and passkey sign-in, usage
analytics and the base console.

Out: AI providers, hosted MCP and the marketplace, agents, agent runs,
notifications, the brain, OpenTelemetry, the bridge surface, cloud deploy,
Slack.

The kernel floor (API server, CLI, runtime, users, OAuth, the gateway, files,
documents, the scheduler and cloud) is linked by every flavour; see
`docs/flavours/README.md`. Because the gateway member is in that floor,
`/v1/models` answers `{"data":[],"has_more":false}` here, and the kernel's
agent proxy answers `/api/v1/agents` (401 without a token).

## Status on core `future` b41c8b1f8

The image builds, but first boot stops in `admin setup` with "At least one
AI provider is required": core requires a provider key whether or not the
composition links the AI member. Three more core refusals follow once that
is lifted: startup validation runs the AI config validator ("No AI providers
are enabled"), server start refuses when no MCP process set is admitted
("McpServer processes are not supervised on this host"), and the gateway's
accounting recovery queries `ai_requests`, a table this composition does
not install. With those four core changes applied, the image boots, passes
`just smoke site` (32 assertions) and serves the console described below.
Until core ships them, `just smoke site --no-build` reports the setup
error.

`just setup site` hits the same `admin setup` refusal: this flavour takes
no provider key.

## Run it locally

```bash
just setup site                             # no provider key
FLAVOUR=site just start
```

Setup builds `target/flavour-site/release/systemprompt`, starts Postgres
from `.systemprompt/docker/site.yaml`, writes `.systemprompt/profiles/site/`,
stages the services subtrees `flavour.yaml` lists into
`.systemprompt/flavours/site/services` (with this directory's `services/`
overrides on top), migrates, creates the bootstrap admin and publishes the
site.

## Run the image

```bash
just docker-build site                      # systemprompt-template:site
ADMIN_EMAIL=you@example.com FLAVOUR=site docker compose -p sp-site up
just smoke site --no-build                  # boot and assert the smoke expectations
just metrics site                           # target/flavour-metrics/site.json
```

The image uses the `slim` runtime. Compose needs `ADMIN_EMAIL` on first
boot and no provider key. Set `EXTERNAL_URL` to the address the browser
uses when it is not `http://localhost:8080`: passkeys are bound to that
origin, and core derives the API URLs, CORS origin and JWT issuer from it.
First boot waits for Postgres, runs `admin setup`, migrates (11 members:
database, events, users, logging, authz, oauth, analytics, files, content,
scheduler, web), creates the bootstrap admin, applies the declared
configuration, publishes the site and starts the server. Later boots reuse
the profile in the `app_profiles` volume.

Measured with the four core changes (24 cores): binary 76.9 MB, image
213.6 MB (81.4 MB gzip, 11 layers), 637 crates, clean image build 626 s,
boot to healthy 11.3 s, a restart 9.1 to 12.5 s, migrating a fresh database
4.2 s, a second `infra db migrate` 1.9 s, idle RSS 28.6 MB.

## First login

The console signs in with a passkey. The bootstrap admin has none, so mint a
one-time link and open it in the browser:

```bash
docker compose -p sp-site exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` URL, choose Register Passkey,
then sign in at `/admin/login` with the same email. The browser lands on
`/admin/profile`.

`systemprompt admin session login` fails on this flavour ("no admitted
member provides DynContextDirectory"): core gives a CLI session a
conversation context, which only the agents member provides.

## Dashboard

`/` is the published site, with `/blog`, `/documentation`, `/sitemap.xml`
(41 URLs from the shipped content), `/robots.txt` and `/llms.txt`.
`/admin` lands on the overview, with the home cards Content, Traffic and
Publishing. The sidebar groups are Traffic (overview, analytics, sessions),
Governance (governance, approvals, secrets audit), People & access (users,
groups, projects, roles, access control, devices), Platform (code sync,
configuration, data lifecycle, observability) and Account (profile,
settings, help).

They come from `flavours/site/services/web/config/dashboard.yaml`, which
overrides `services/web/config/dashboard.yaml`. Naming an item whose page
this flavour does not compile refuses boot with the console feature it
needs.

Three of those pages answer 500 on this flavour: Users, Configuration and
Observability. Their queries read the gateway's `ai_requests`,
`ai_gateway_policies` or MCP tables, which this flavour does not install.
Analytics, Sessions, Governance, Groups, Projects, Roles and Access control
render, but their AI-usage panels are empty and log the missing tables.

## CLI groups

`admin`, `analytics`, `build`, `core`, `infra`, `plugins` and `web`.
`cloud` and `admin agents` are not compiled in.

## Configuration

`profile.overlay.yaml` sets nothing. The services overrides:

- `services/scheduler/config.yaml` lists only the jobs this flavour
  compiles. Core refuses to boot on a configured job that no member
  contributes.
- `services/config/config.yaml` includes only the files of the subtrees
  this flavour ships.
- `services/access-control/rules.yaml` declares no entity. Every shared
  entity is a marketplace, skill, agent, MCP server or gateway route.
- `services/web/config/dashboard.yaml` is the console layout above.
