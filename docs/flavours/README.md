# Flavours

A flavour is a named build of this template: one cargo feature on the root
crate (`flavour-<name>`), the binaries it ships, the parts of `services/` it
loads, and what its image and local install do at boot. Every flavour is a real
feature subset: a capability a flavour does not name is not compiled in, so its
routes, CLI groups, jobs and tables are absent rather than switched off.

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

| Flavour | Cargo feature | Facade features on top of `core,runtime,identity,analytics` | Template members |
|---|---|---|---|
| `gateway` | `flavour-gateway` | `ai`, `cloud`, `otel`, `bridge` | web (console `gateway`) |
| `mcp` | `flavour-mcp` | gateway's + `mcp-servers`, `marketplace` | web (`gateway`, `mcp`), the `systemprompt` MCP toolset |
| `agents` | `flavour-agents` | mcp's + `agents`, `agent-runs`, `notifications`, `brain` | web (`gateway`, `mcp`, `agents`), the MCP toolset |
| `site` | `flavour-site` | `content`, `files`, `files-pdf` (no `ai`) | web (`content`: public site, publish pipeline) |
| `full` | `flavour-full` | `reference` + `slack` | everything |

- `gateway`: an AI gateway only: provider routing, policies, quotas, audit and analytics, for teams that point existing clients at a governed endpoint.
- `mcp`: hosted MCP servers and the marketplace, governed per user, with no agents and no public site.
- `agents`: agents and agent runs on top of the gateway and MCP hosting, with notifications and organisational memory.
- `site`: the public site and content pipeline (pages, documentation, sitemap) with files and PDF support, no AI.
- `full`: everything above plus the whole admin console and Slack.

Cargo features are additive, so a build needs at least one `flavour-*`
(`src/lib.rs` refuses none) and each larger flavour includes the one below
it: `agents` turns on `mcp`, which turns on `gateway`. `full` is the union:
`agents` + `site` + `reference` + Slack. Naming two unrelated flavours
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

## The kernel floor

Every flavour links the kernel: the API server, the CLI, the runtime and MCP
host, and the members they depend on directly (users, OAuth, AI, the gateway,
files, documents, the scheduler, agent runs, issues and cloud). A flavour can
leave out the agent and MCP client crates, content with the generator and
templates, the marketplace, organisational memory, notifications, office, PDF
and collaborative-file support, OpenTelemetry, Slack and Teams, and the
template's own admin page groups. Shrinking the floor itself needs optional
seams in the kernel; until then the size difference between flavours is
bounded by it, and the metrics show by how much.

## Schema per flavour

The web extension's schema is a feature-gated contribution, the same rule
core follows: a schema file whose objects read an optional member's table is
declared in `extensions/web/src/schemas.rs` only under the web feature that
links that member (`gateway`: request scopes, the `ai_requests` audit
trigger and request-log expiry; `mcp`: skill version impact,
`tool_activity`, `user_last_seen` and marketplace binding expiry; `agents`:
`conversation_requests` and the conversation facts; `content`: the
`markdown_content` side table). No stand-in view replaces an absent table,
and no DDL tests what is installed. Console pages and jobs that read those
objects are compiled with the same feature, so each flavour installs one
schema and `full` installs the union. Every flavour's web schema installs on
a database holding only its members' tables (checked file by file against
each flavour's table set).

The console follows the same rule at the query. A repository module, a page
section or a keyed read that names an optional member's table is compiled
only under the admin feature that links that member, so a base page reads its
base and the member adds its part: the user roster and its KPIs, the user
detail headline and usage tab, the profile's usage cards, roles, the overview
and the governance safety tab each render without the gateway member. Pages
that are only a member's figures are compiled out with their nav item: the
analytics dashboard and the group pages need `gateway`, the project pages
`mcp` (they read requests beside tool executions), the conversation surfaces
`agents`. `scripts/check-flavour-queries.sh <flavour|all>` (`just
check-flavour-queries`) proves it: it migrates a fresh database with each
flavour's binary and re-checks the template's sqlx crates online against it,
so a query naming a relation the flavour does not install fails at its line.

## Signing in to the console

Every flavour signs in the same way. The console uses passkeys, and a fresh
install's bootstrap admin has none, so issue a one-time link:

```bash
docker compose -p sp-<flavour> exec app systemprompt admin users webauthn \
  generate-setup-token --email you@example.com
```

Open the printed `/auth/link-passkey?token=…` link (valid for 15 minutes) on
the host `EXTERNAL_URL` names, register the passkey, then sign in at
`/admin/login`. Smoke signs in the same way with a software authenticator
(`scripts/lib/console-login.py`) and asserts each flavour's
`smoke.console_pages` (200) and `smoke.routes_absent_signed_in` (404).
The developer login link is removed (nothing issued its codes, and an
install member cannot mint a browser session). AD FS single sign-on still
verifies an assertion but is refused at session minting on this core; it
needs to be contributed through core's `SsoSlot` to work.

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

## Metrics

`scripts/flavour-metrics.sh <flavour|all>` (`just metrics`) writes
`target/flavour-metrics/<flavour>.json` and `target/flavour-metrics/table.md`.
Each run records the core revision, template commit, rustc version and host
cores and memory.

- `--image`: stripped binary bytes, image bytes uncompressed and gzipped,
  layer count, crates in the dependency closure (`cargo tree`), clean and
  cached image build seconds, migrate seconds on a fresh database,
  boot-to-healthy seconds and idle memory 30 seconds after healthy.
  `--bench` adds `hey` p50/p99 against the health endpoint.
- `--dev`: wall time of `cargo check` and `cargo clippy --all-targets -- -D
  warnings`, each from a clean target directory and incrementally after
  touching `src/lib.rs` and `extensions/web/admin/src/lib.rs`; compile and
  run time of the `tests/` workspace; the 15 slowest units of a clean build.
  Cargo commands run one at a time; `FLAVOUR_METRICS_JOBS` sets `-j`.
- `--compare <old.json>`: before/after deltas for every number, so an
  optimisation is recorded as a pair.
- `--table`: measure nothing; rebuild `table.md` from the per-flavour JSON.
  Each row names the core and template commit it measured.

While the root builds against a patched core and `tests/` still resolves
systemprompt from crates.io, `--dev` records the `tests/` steps as
`"status": "not available"` with the reason, not as a time.

The `Flavours` workflow builds, measures and smokes every flavour and attaches
the JSON and table to the run. Nothing generated is committed; the release
copies `target/flavour-metrics/table.md` here.
