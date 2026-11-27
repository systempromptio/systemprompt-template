# Flavours

A flavour is a named build of this template: one cargo feature on the root
crate (`flavour-<name>`), the core composition it links, the binaries it
ships, the parts of `services/` it loads, and what its image and local
install do at boot. Every flavour is a real feature subset: a capability a
flavour does not name is not compiled in, so its routes, console pages, CLI
groups, jobs and tables are absent rather than switched off.

`flavours/index.yaml` lists the flavours in order. Each has a directory:

| File | Holds |
|---|---|
| `flavours/<name>/flavour.yaml` | description, `cargo_feature`, `extra_features`, `binaries`, the shared `services` subtrees, `requires_ai_key`, `site`, `runtime` (`slim` or `full`) and the `smoke` expectations |
| `flavours/<name>/profile.overlay.yaml` | profile keys set on top of what `admin setup` writes |
| `flavours/<name>/services/` | overrides copied over the shared subtrees; every subset flavour overrides `scheduler/config.yaml` (only the jobs it compiles: core refuses to boot on a configured job no member contributes) and `config/config.yaml` (only the includes of the subtrees it ships) |
| `flavours/<name>/README.md` | how to run it |

`scripts/flavour.py` is the only reader of these files (`list`, `table`,
`get <name> <key>`, `features <name>`, `matrix`, `check`); every script,
recipe and workflow asks it. It runs on any `python3` with PyYAML
(`python3-yaml`), which the setup scripts already need.

## The flavours

Each template flavour mirrors core's line of the same name in
`scripts/build-compositions.txt`; the members are that line's entry in core's
`scripts/composition-members.txt`.

| Flavour | Cargo feature | Core facade features | Core members (beside the kernel's `database`, `events`, `users`, `logging`, `authz`, `oauth`, `identity`, `mcp_host`, `scheduler`) |
|---|---|---|---|
| `gateway` | `flavour-gateway` | `core,runtime,identity,ai,analytics,cloud,otel,bridge` | `ai`, `analytics`, `gateway`, `analytics-http`, `gateway-http`, `identity-http`, `cloud` |
| `mcp` | `flavour-mcp` | gateway's + `mcp-servers`, `marketplace` | gateway's + `mcp`, `managed_resources`, `marketplace-http`, `mcp-http` |
| `agents` | `flavour-agents` | mcp's + `agents`, `agent-runs`, `notifications`, `brain` | mcp's + `agent`, `brain`, `notifications`, `agent_runs`, `notifications-http`, `agent_runs_api`, `agent-http`, `brain-http` |
| `site` | `flavour-site` | `core,runtime,identity,analytics,content,files,files-pdf` | `analytics`, `documents`, `files`, `content`, `issues`, `generator`, `content-http`, `files-http`, `analytics-http`, `identity-http` |
| `full` | `flavour-full` | core's `reference` composition + `slack` | every member of `reference`, plus `slack` |

What each serves, and what it deliberately leaves out:

- `gateway`: the AI gateway (`/v1/messages`, `/v1/chat/completions`,
  `/v1/responses`, `/v1/models`), gateway policies and quotas, audit and
  usage analytics, the bridge device surface, OpenTelemetry ingest, cloud
  deploy and the base console with its gateway pages (overview, analytics,
  requests, gateway, policies, quotas, users, groups, observability). No
  hosted MCP, marketplace, agents or public site; no sessions,
  conversations or history pages (they read the conversation record, which
  only a build with agents installs) and no project pages.
- `mcp`: gateway's, plus hosted MCP servers (the `systemprompt` toolset)
  and the marketplace (plugins, skills, marketplaces, managed resources),
  governed per user, with project pages. No agents, no public site, and,
  as in `gateway`, no sessions, conversations, history or version-analysis
  pages.
- `agents`: mcp's, plus agents, agent runs, notifications and
  organisational memory, and the conversation surfaces (sessions,
  conversations, history, traces, version analysis). No public site.
- `site`: the public site and content pipeline (pages, blog,
  documentation, sitemap, `robots.txt`, `llms.txt`), files and PDF export,
  and the base console (overview, users, governance, configuration,
  observability). No AI: no `/v1/models` (the composition has no gateway
  member), no requests or analytics pages, no group or project management
  pages, no conversation pages, no cloud CLI group.
- `full`: everything above, the whole admin console and Slack.

Cargo features are additive, so a build needs at least one `flavour-*`
(`src/lib.rs` refuses none) and each larger flavour includes the one below
it: `agents` turns on `mcp`, which turns on `gateway`. `full` is the union
of `agents`, `site`, `reference` and Slack. Naming two unrelated flavours
builds their union. Without `content` the template serves no public site
and `/` redirects to the console.

The `systemprompt-kit-export` developer binary uses the marketplace and so
belongs with `mcp`, `agents` and `full`; it is built on demand by
`just kit-export` and is in no image.

## Choosing a flavour

Start from what clients will reach. Only model traffic: `gateway`. Tools for
clients you already run: `mcp`. Autonomous work: `agents`. A published site:
`site`. An evaluation of the whole platform, or the demos in `demo/`: `full`.
`requires_ai_key` says whether setup asks for a provider key; `site` says
whether the publish pipeline runs.

## Rules the tree follows

**Schema contributions are feature-gated.** A web schema file whose objects
read an optional member's table is declared in `extensions/web/src/schemas.rs`
only under the web feature that links that member (`gateway`: request
scopes, the `ai_requests` audit trigger and request-log expiry; `mcp`: skill
version impact, `tool_activity`, `user_last_seen` and marketplace binding
expiry; `agents`: `conversation_requests` and the conversation facts;
`content`: the `markdown_content` side table). No stand-in view replaces an
absent table, and no DDL tests what is installed. Each flavour installs one
schema and `full` installs the union.

**Queries are checked per flavour.** A repository module, page section or
keyed read that names an optional member's table is compiled only under the
admin feature that links that member, so a base page reads its base and the
member adds its part. Pages that are only a member's figures are compiled
out with their nav item. `scripts/check-flavour-queries.sh <flavour|all>`
(`just check-flavour-queries`) migrates a fresh database with each flavour's
binary and re-checks the template's sqlx crates online against it, so a
query naming a relation the flavour does not install fails at its line.

**Absent fields are typed.** A view-model field whose member a flavour does
not link stays in the struct as `types::Absent` (serialised `null`), and the
template branches on it with `{{#if}}`.

**Links go only to compiled pages.** A link to another console page is drawn
only when that page is compiled, read from the nav registry that builds the
sidebar: templates branch on `pages.<nav id>` (`dashboard::PageLinks`, in
every page shell, `null` for a page this build leaves out), and view models
take their hrefs from `dashboard::page_href`, `page_url` or `link_to`, which
answer `None` for the same pages. A `dashboard.yaml` naming a nav item the
flavour does not compile refuses boot.

**Smoke proves it on the image.** `scripts/flavour-smoke.sh` boots the image
and asserts the flavour's `smoke` block: routes present and absent, CLI
groups, capabilities, MCP servers and `/v1/models`. It then signs in with a
passkey (a software authenticator, `scripts/lib/console-login.py`), seeds the
rows a fresh install lacks through the CLI and the console API only (a second
user, a group and a project holding them, and, where the gateway is
compiled, one gateway request whose `ai_requests` row must be recorded),
asserts `smoke.console_pages` (200) and `smoke.routes_absent_signed_in`
(404), fetches every page the rendered sidebar links (200) and every console
link those pages carry, one hop deep (neither 404 nor 5xx), with
`scripts/lib/console-crawl.py`. A strict-mode template reading a field the
view model does not supply answers 500 there, and a link to a page the
flavour does not compile answers 404.

## Signing in to the console

Every flavour signs in the same way: with a passkey. A fresh install's
bootstrap admin has none, so issue a one-time link:

```bash
docker compose -p sp-<flavour> exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

For a local install, run the same `systemprompt admin users webauthn
generate-setup-token` command with `target/flavour-<flavour>/…/systemprompt`
and `--profile <flavour>`. Open the printed `/auth/link-passkey?token=…` link
(valid for 15 minutes) on the host `EXTERNAL_URL` names (the passkey is
bound to that host), register the passkey, then sign in at `/admin/login`.
There is no other browser sign-in. AD FS single sign-on verifies an
assertion but is refused at session minting on this core; it needs to be
contributed through core's `SsoSlot` to work.

## Using a flavour

```bash
just flavours                                    # what exists
just setup full --anthropic-key <key>            # local install
FLAVOUR=full just start                          # run it (FLAVOUR defaults to full)
just docker-build full                           # image systemprompt-template:full
just smoke full --no-build                       # boot it and assert smoke
just metrics full --image --dev                  # measure it
```

Every build, run and test recipe reads `FLAVOUR` (default `full`): builds go
to `target/flavour-<name>/`, the profile is `.systemprompt/profiles/<name>/`
and its Postgres is `.systemprompt/docker/<name>.yaml`, so two flavours never
invalidate each other's build or share a database.

## Adding a flavour

1. Add the cargo feature `flavour-<name>` to the root `Cargo.toml` and make
   `cargo check --no-default-features --features flavour-<name>` and clippy
   clean.
2. Create `flavours/<name>/` with `flavour.yaml`, `profile.overlay.yaml` and
   `README.md` (copy `flavours/full/` and narrow it), and add the name to
   `flavours/index.yaml`.
3. Write `flavours/<name>/services/web/config/dashboard.yaml` (landing,
   home cards, nav sections) from the nav item ids in
   `extensions/web/admin/src/dashboard/registry/`; an id the flavour does not
   compile refuses boot, so the file is checked the first time it starts.
4. Run `bash scripts/check-flavours.sh`, then `just docker-build <name>`,
   `just smoke <name> --no-build` and `just metrics <name>`.

## Core pin

Every flavour builds against one systemprompt-core commit, pinned by git rev.
`bridge/CORE_REF` holds the SHA; every core crate `Cargo.toml` and
`tests/Cargo.toml` name is a git dependency
`{ version = "X.Y.Z", git = "https://github.com/systempromptio/systemprompt-core", rev = "<sha>", … }`
(no `[patch]` table), so each lockfile holds one copy of every core crate, and
both lockfiles are committed resolved against it. A clone builds every
flavour, and `docker build` fetches core itself (cached in `~/.cargo/git`),
with no sibling checkout.

- Move the pin: `just core-pin <sha|branch>` (`scripts/core-pin.sh`), then
  commit `bridge/CORE_REF`, both manifests and both lockfiles.
- Check it: `scripts/core-pin.sh --check` (part of `scripts/check-core-ref.sh`).
- Build against a local core checkout: `scripts/core-patch.sh on <checkout>`
  appends a `[patch."<core url>"]` table of path entries and hides the
  manifests and locks from git; `scripts/flavour-build.sh` then passes the
  checkout as the `core` build context. `scripts/core-patch.sh off` restores
  the pin, and `status` prints `pin` or `path`. The tracked pre-commit hook
  (`just init-hooks`) refuses a staged `[patch]` path entry.

## The image

`Dockerfile` builds one image per flavour (`just docker-build <flavour>`,
tag `systemprompt-template:<flavour>`).

- **Base.** The runtime is `gcr.io/distroless/base-nossl-debian12` plus
  busybox, jq, lsof, setpriv and the shared objects they and the binary need
  (`docker/runtime-tools.sh`). There is no Python, curl, gosu or openssl at
  runtime. The entrypoint (`docker/entrypoint.sh`) is POSIX sh, and the
  healthcheck (`docker/healthcheck.sh`) is an HTTP GET of `/api/v1/health`
  over busybox `nc`. The server runs as the unprivileged `app` account.
  `runtime: full` (the `full` flavour) adds `demo/` and `hey`.
- **Per-flavour content.** Only the flavour's services tree is staged; the
  video assets ship only with a site, and the hosted plugin bundles only
  when the flavour ships `plugins/`. The flavour overlay is merged with
  `flavours/profile.base.yaml` at build time and written to
  `<profiles>/base.yaml` on every boot.
- **First boot.** The entrypoint waits for Postgres, runs `admin setup`
  against `DATABASE_URL`, migrates, creates the bootstrap admin for
  `ADMIN_EMAIL`, applies the config planes and publishes (the site, or the
  console assets). Later boots reuse the profile in the persisted volume.
- **The broker secret.** Every flavour's composition installs the web
  member, which serves the connector token route (the `broker-calls`
  conferral), so every flavour's server needs `mcp_credential_broker_secret`
  in its secrets. `just setup` and the image's first boot mint it once and
  keep it. A supplied profile (`SYSTEMPROMPT_PROFILE_DIR`: Helm, air-gap,
  scaled) must carry it, as it carries `encryption_master_key`; the
  entrypoint refuses to start without it, because a per-replica value would
  fail broker verification across replicas.

## The kernel floor

Every flavour links the kernel: the API server, the runtime, the MCP host
and the CLI, with the members they need (`database`, `events`, `users`,
`logging`, `authz`, `oauth`, `identity`, `mcp_host` and `scheduler`; core's
`runtime` line). The kernel links no capability: the gateway, files,
documents and every other capability come only with the facade feature
that names them. `ai` brings the gateway members (`ai`, `gateway`,
`gateway-http`); `files` brings files, documents and issues. A template
page group or job that reads a capability compiles only under the feature
that links it.

Crates in each flavour's dependency closure (normal and build), before and
after the floor was cut:

| flavour | before | after |
|---|---|---|
| gateway | 616 | 569 |
| mcp | 625 | 578 |
| agents | 631 | 586 |
| site | 637 | 596 |
| full | 724 | 715 |

On the pinned core (`f61601289`) every flavour's smoke passes with no
failure: gateway 227, mcp 298, agents 400, site 138 and full 397 assertions.
CI on template `abfa6d3e` is green.

## Metrics

`scripts/flavour-metrics.sh <flavour|all>` (`just metrics`) writes
`target/flavour-metrics/<flavour>.json` and `target/flavour-metrics/table.md`.
The measured table for the current pin is [metrics.md](metrics.md); each row
names the core and template commit it measured, with the rustc version and
host cores and memory.

Columns:

- **binary**: stripped `systemprompt` bytes.
- **image / gzip**: image bytes uncompressed and gzipped; **layers**: layer count.
- **crates**: crates in the dependency closure (`cargo tree`, normal + build).
- **build clean / cached**: image build seconds from an empty and a warm cache.
- **migrate**: seconds to migrate a fresh database.
- **boot**: seconds from start to healthy; **idle RSS**: memory 30 seconds after healthy.
- **check / clippy**: wall time of `cargo check` and `cargo clippy --all-targets
  -- -D warnings`, from a clean target directory and incrementally after
  touching `src/lib.rs` and `extensions/web/admin/src/lib.rs` (`--dev`).
- **tests**: compile and run time of the `tests/` workspace (`--dev`); without
  `SYSTEMPROMPT_TEST_DATABASE_URL` it reads `not available` with the reason,
  never a time.

Modes: `--image` (image, boot and memory figures; `--bench` adds `hey`
p50/p99 against the health endpoint), `--dev` (build and test timings; cargo
commands run one at a time, `FLAVOUR_METRICS_JOBS` sets `-j`), `--compare
<old.json>` (before/after deltas for every number), `--compare-images OLD NEW`
(bytes, gzip bytes, layers and binary bytes of two images side by side) and
`--table` (rebuild `table.md` from the per-flavour JSON).

Image sizes measured so far (uncompressed). The Debian slim column is the
image before the runtime moved to distroless; the distroless column was
measured with `--compare-images` on core `future` snapshot `b41c8b1f8` and the
template's distroless change on top of `d9ec34a8`. The site figure on the
distroless base was measured separately from the other four.

| flavour | Debian slim base | distroless base |
|---|---|---|
| gateway | 206 MB | 92 MB |
| mcp | 212 MB | 102 MB |
| agents | 226 MB | 117 MB |
| site | 214 MB | 101 MB |
| full | 266 MB | 157 MB |

The `Flavours` workflow builds, measures and smokes every flavour and
attaches the JSON and table to the run.
