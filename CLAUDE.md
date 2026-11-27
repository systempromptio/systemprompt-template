# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Enterprise Demo

**Use the CLI to discover commands.** `systemprompt --help` is your starting point.

---

## Branching (this repo)

**All work lands on `next`. Never push to `main`.** `next` is the default
branch; `main` is protected by a ruleset (pull request only, no bypass for
anyone) and only moves through a frozen promotion PR that `just release`
opens. Full contract: `docs/BRANCHING.md`; procedure: `docs/RELEASING.md`.

```
next   ← default branch. Push freely; gates.yml runs the whole gate on every push.
  ↓ `just release X.Y.Z` — freezes the exact green next commit on
  ↓ promote/X.Y.Z/<main>/<sha>, opens the PR; run it again once the PR's
  ↓ proof is green to merge and push the vX.Y.Z tag
main   ← release-only. Every commit is a verified promotion merge with a tag.
```

**Versions are in lockstep with core**: core `X.Y.Z` → workspace `X.Y.Z` →
tag `vX.Y.Z` → image `:X.Y.Z` → Helm `appVersion`.
`scripts/sync-release-version.sh X.Y.Z [--check]` owns the product pins and
refuses a release off core's `MAJOR.MINOR`; `scripts/sync-core-version.sh
X.Y.Z [--check]` owns every core crate pin in **both** workspaces
(`Cargo.toml`, `tests/Cargo.toml`) and `bridge/CORE_REF`. `just core-bump X.Y.Z`
runs both, refreshes both lockfiles, migrates the local DB
(`--profile local`) and builds.

**Core is pinned by git rev.** `bridge/CORE_REF` holds one 40-char SHA of
https://github.com/systempromptio/systemprompt-core; every core crate
`Cargo.toml` and `tests/Cargo.toml` name is a git dependency
`{ version = "X.Y.Z", git = "<core url>", rev = "<sha>", … }` (no `[patch]`
table), so core's inter-crate dependencies resolve inside that one git source
and each lockfile holds exactly one copy of every `systemprompt-*` crate. Both
lockfiles are committed resolved against it, so a fresh clone builds with no
sibling checkout. Moving the pin is one command: `just core-pin <sha|branch>`
(`scripts/core-pin.sh`: writes CORE_REF, rewrites every entry in the root,
`tests/` and `extensions/mobile/core` manifests, re-resolves their locks). `scripts/core-pin.sh --check` (inside `scripts/check-core-ref.sh`,
a lint gate) fails on another rev, a registry or duplicate core crate in a
lock, or a `[patch]` table. Push the core commit before pinning it.

**Local core override.** `scripts/core-patch.sh on <core checkout>` appends a
`[patch."<core url>"]` table of path entries (one per core crate each lockfile
resolves) to the three manifests and hides the six
manifests/locks from git (skip-worktree); `scripts/core-patch.sh off` restores
the committed pin; `status` prints `pin` or `path`. While it is on,
`scripts/flavour-build.sh` passes the checkout as the `core` BuildKit context
and `just deploy` (`core-guard`) refuses. `.githooks/pre-commit`
(`just init-hooks`) refuses a staged `[patch]` path entry of either form and a
pin that disagrees with CORE_REF. Do not run `just prepare` while an override
is on. `main` never carries a core git dependency or a `[patch]` table
(`scripts/validate-release.sh`).

**Core (`../systemprompt-core`) is write-only from here.** Commit core changes
on core's `next` and push; run no validation in the core checkout — this repo
compiles the pinned core anyway.

## Quick Start

```bash
# Flavours (flavours/index.yaml, docs/flavours/README.md): every build, run and
# test recipe targets FLAVOUR (env, default full). scripts/flavour.py reads the
# definitions with any python3 that has PyYAML.
just flavours                                                             # name + description

# First-time setup of a flavour: writes .systemprompt/profiles/<flavour>/, starts
# Docker Postgres, builds into target/flavour-<flavour>/, migrates, bootstraps and
# (for a flavour with a site) publishes. With no key, the CLI prompts for the
# provider; passing keys is non-interactive — the first becomes the default.
just setup full                                                           # interactive provider pick
just setup full --anthropic-key <k> [--openai-key <k>] [--gemini-key <k>] [--http-port 8080] [--pg-port 5432]

# Image, smoke and metrics per flavour
just docker-build full && just smoke full --no-build && just metrics full --image --dev

# No toolchain? Install the release's binaries into target/flavour-full/release/ instead
just fetch-release [X.Y.Z]                                                # linux-amd64/arm64, darwin-arm64

# Build (locked dependencies, SQLX_OFFLINE=true; no database changes)
just build            # debug
just build --release  # release

# Lint (all workspace targets, locked dependencies, SQLX_OFFLINE=true, -D warnings)
just clippy

# Regenerate .sqlx/ offline query cache (needs live DB)
just prepare

# Start services
just start

# Discover CLI commands
systemprompt --help

# List skills
systemprompt core skills list
```

---

## Shared Build State (read this before you compile)

Several agents work this clone at once. Builds, clippy, and tests are expensive
and take a shared cargo lock, so a build started mid-iteration stalls everyone.

**Do all the work first, validate once at the end.** Never run `just build` or
`just clippy` between edits to see how you're doing; finish the change set, then
run the gate a single time.

**Check the shared state before spending anything:**

```bash
just build-status     # in-flight run + last result per recipe (a record, never a reason to skip)
just server-status    # running server, its binary, and whether that binary is stale
```

`just build`, `just clippy`, `just test-*`, `just doc-check`, `just msrv-check`,
`just coverage` and `just lint-gates` are single-flight
(`scripts/build-coordinator.sh`). One rule: **one build in flight at a time,
always of the latest source.** Every call compiles the tree as it is (cargo is
incremental, so an unchanged tree costs seconds); there is no "already built,
skip" cache — a skip cache can report green while a bare `cargo build` from
another tree has overwritten `target/debug/systemprompt` with stale code.

| situation | what happens |
|-----------|--------------|
| a run of this exact source is in flight | you are told so, attach to its log, exit with its status |
| a run of different source is in flight | you are told so, wait for it, then run over the latest tree |
| nothing in flight | you lead |

**Free-disk guard.** Before it starts a compile (and again when it takes the
lock after waiting), the coordinator refuses if the volume holding `target/`
(`CARGO_TARGET_DIR` when set) has less than **`BUILD_MIN_FREE_GB`** GB free —
default **25**. A debug `target/` here is tens of GB and a build that runs the
disk dry fails as an unrelated linker error while taking every other process
on the machine down with it; that happened on 2026-09-28. Free space first
(`cargo clean`, old worktrees' `target/`, `coverage-report/`), or lower the
threshold deliberately (`BUILD_MIN_FREE_GB=15 just build`; `0` disables it).
`lint-gates` is exempt (read-only), and `BUILD_NO_COORD=1` (CI) bypasses the
coordinator and the guard together.

Results land in `.build/` (gitignored): `runs.jsonl`, `latest/<recipe>.json`,
`logs/`, `binaries.jsonl`. Read them instead of re-running.

`just start` reports the running server first, then starts. It does not restart
a server another agent is already running (say so and stop), and it warns when
the binary predates the current source, but it only refuses outright when there
is no binary at all. Staleness is reported from the ledger when the binary came
from a coordinated build, and from file mtimes otherwise. `just stop` shuts
this clone's services down cleanly.

Always go through the justfile. A bare `cargo build` bypasses coordination and
re-creates the contention. Escape hatches when you truly need them:
`START_FORCE=1`, `BUILD_NO_COORD=1` (`BUILD_FORCE=1` is accepted and does
nothing: every run already compiles).

---

## Preflight (the gates; CI re-runs them on every push)

```bash
just verify             # what gates.yml runs: preflight-static → preflight-lint → test
just preflight          # verify + coverage floor/ratchet
just preflight-static   # no compile: fmt, sqlx cache, version/core pins, release self-tests, deploy config, docker entrypoint syntax, lint gates
just preflight-lint     # clippy -D warnings, rustdoc -D warnings, MSRV
just preflight-full     # weekly: preflight + deny + audit + machete + hack (both workspaces)
just e2e-gate           # the browser tier; needs a running stack (`just start`)
just init-hooks         # once per clone: tracked .githooks/ (pre-commit only)
```

Each collector runs every check even after one fails and reports them all.
`.github/workflows/gates.yml` runs the same tiers independently on every
`next` push and ordinary PR — static, lint, test (Postgres 18), e2e
(Playwright), and supply-chain (cargo-deny, whose advisories check is the one
advisory source, and cargo-machete, both over the root and `tests/`
workspaces) — and a single **`Gates passed`**
job that fails unless every tier succeeded; it is the context the `main`
ruleset should require. Frozen promotion PRs skip the tiers and verify the
exact next-push proof instead.

`just lint-gates` runs the `gates=()` array in the justfile concurrently
(trust the array, not a count). Every failure is reported. Two of them,
`check-fail-open` and `check-discarded-results`, run core's rust-contracts
scanner from the sibling checkout: locally they skip with a message when
`../systemprompt-core` is absent (`just core-checkout` clones it at
CORE_REF); in CI its absence fails them. A deliberate discard carries
`// Why: discard-ok: <reason>` on the line above it. Known unresolved template
fields live in `scripts/template-fields-exemptions.txt` and unreferenced
assets in `scripts/asset-reachability-exemptions.txt`; both fail on stale
entries, so the lists only shrink.

**Coverage floor + ratchet.** `just coverage` runs an instrumented llvm-cov
pass over both workspaces (root, `tests/`) into `coverage-report/`
(gitignored); `just coverage-check` enforces the tracked
`coverage/baseline.json` — a floor, a 0.5pt total ratchet and per-crate
ratchets — and refuses to record a baseline under half the previous total (a
run that lost its instrumented binaries reports 0.00%). Raise it with
`just coverage-baseline`, then `just coverage-badge`; the `coverage-badge.sh`
gate fails if the README badge and the baseline disagree. Never use
cargo-llvm-cov (see `scripts/coverage.sh`). `coverage.yml` measures `main`
and nightly `next`; it does not gate releases. No baseline is tracked yet, so
CI keeps one per ref in the Actions cache (`COVERAGE_BASELINE`,
`COVERAGE_ESTABLISH`, `COVERAGE_RATCHET` in `scripts/coverage-check.sh`): the
first run reports and establishes it, later runs ratchet against it and only
ever raise it. A committed `coverage/baseline.json` takes over from it.

**The schema-upgrade ladder.** `tests/fixtures/schema/release-baseline-<X.Y.Z>.sql`
holds one recorded schema per release from the floor (0.61.0) up;
`scripts/check-schema-baseline.sh` requires a rung for every release tag from
the floor and one named for the workspace version, and `just release`
refuses without it. `just schema-baseline` records the current tree's rung;
`just schema-baseline X.Y.Z` records a published release's from its gateway
tarball. The release pipeline's `upgrade-boot` boots the candidate image over
every rung seeded by `seed_hot_tables.sql`, and
`tests/integration/schema-upgrade` restores each rung, seeds it the same way,
runs the current installer over it (2000 rows per hot table must survive) and
diffs the result against a fresh install. Never edit a rung by hand.

**Schema retirements and `@supersedes-checksum`.** Migrations are history:
never edit or renumber an applied one. To retire a trigger, function,
procedure or view another extension's migration would trip over, add
`extensions/web/schema/retire/NNN_<name>.sql` (only `DROP TRIGGER|FUNCTION|
PROCEDURE|VIEW … IF EXISTS`; core refuses anything else) and override
`Extension::retirements()` in `extensions/web/src/extension_impl.rs` to return
it; core runs every extension's retirements before any migration. None exist
yet, so the default (empty) is inherited and `schema/retire/` holds only a
`.gitkeep`. When an applied migration's text truly must change (it can no
longer replay on a newer core), keep its slot, rewrite it and put
`-- @supersedes-checksum: <old 16-hex checksum>` in its leading comment block;
core accepts exactly that transition instead of refusing boot. A migration
that cannot be replayed at all becomes `NNN_<name>.tombstone` (see 053) — a
number is never reused.

---

## CLI Structure

```
systemprompt <domain> <subcommand> [args]
```

| Domain | Purpose |
|--------|---------|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build core workspace and MCP extensions |

**Use `systemprompt <domain> --help` to explore any domain.**

---

## CLI Discovery Workflow

When you need to perform a task, use the CLI help to find the right command:

```bash
# Top-level help
systemprompt --help

# Domain help
systemprompt core --help
systemprompt infra --help

# Subcommand help
systemprompt core skills --help
systemprompt core skills show --help
```

---

## Architecture (big picture)

- `src/main.rs` is a thin entry point that delegates to the published `systemprompt` core crates (pinned by git rev as git dependencies; `scripts/core-patch.sh on <checkout>` swaps in a local checkout). All customization is **compile-time** via the [`inventory`](https://docs.rs/inventory) crate — there is no dynamic plugin loader.
- Rust code lives in `extensions/`: `extensions/mcp/*` for MCP server extensions, `extensions/web` for page data and template rendering. Each MCP extension has its own crate with `Cargo.toml` + `.sqlx/` offline cache.
- Configuration is YAML under `services/`, loaded through `services/config/config.yaml`'s explicit `includes:` list. Unknown keys error loudly (`#[serde(deny_unknown_fields)]`).
- Governance runs as a four-stage synchronous pipeline on every tool call: **scope check → configured secret scan → blocklist → rate limit**. Every decision is audited to Postgres with a trace_id linking identity → agent → tool → result → cost. The broad starter signature catalog lives in `services/governance/config.yaml`; forks own their final catalog.
- Per-clone Docker Postgres: `just db-up / db-down / db-logs [tenant=local]`. Project name is derived from a hash of the repo path, so multiple clones on one host get isolated containers and volumes. There is no destructive reset recipe — recover migration checksum drift in place with `just repair-migrations`.
- Deploy flow: `just build-all` (release binary + MCP servers + web assets) then `just deploy`. The `publish_pipeline` job also runs automatically at server startup.

---

## Mobile

The phone apps are the `mobile` flavour (`artefact: mobile`). The apps are
Kotlin and Swift over one Rust binding library, `extensions/mobile/core`: a
cargo workspace of its own (its own `Cargo.lock`, the root lint table
copied), excluded from the root workspace. `just build`, `just clippy` and
the server image never touch any of it. Build that crate with
`CARGO_TARGET_DIR` inside `extensions/mobile/core/target`, as the scripts do.

```
extensions/mobile/android/   Gradle project (Kotlin DSL, wrapper committed), :app, Jetpack Compose
extensions/mobile/ios/       Mobile.xcodeproj (scheme Mobile), SwiftUI
extensions/mobile/core/      the binding crate: links core's systemprompt-mobile at bridge/CORE_REF, adds the fork's own exports
extensions/mobile/shared/    toolchain.env (every pin), env.sh (the Rust library step), build-android.sh, build-ios.sh, stub-instance.py, screens.yaml
flavours/mobile/             the flavour entry
packaging/mobile/            signing templates only; no key is ever tracked
scripts/mobile/              toolchain.sh, android-sdk.sh, rust-targets.sh, android-emulator.sh, ios-tools.sh
```

```bash
just mobile-toolchain         # pins, what this machine has, pin drift
just mobile-android-library   # cargo ndk for three ABIs into jniLibs, then the generated Kotlin
just mobile-android-check     # rustfmt + clippy on core/; Android lint, ktlint, detekt, JVM unit tests (warnings are errors)
just mobile-android           # debug APK
just mobile-android-install   # adb install + launch on the connected device or emulator
just mobile-android-emulator  # headless emulator from the pinned image (stop: ... stop)
just mobile-android-test      # instrumented tests against shared/stub-instance.py
just mobile-ios               # simulator build, unsigned
just mobile-ios-check         # SwiftLint + swift-format
```

The generated Kotlin (`android/app/build/generated/uniffi/`) and the `.so`
files (`android/app/src/main/jniLibs/`) are build output, never committed;
the Android app does not compile until `build-android.sh` has made them. The
core rev the library links moves with `just core-pin`, never by hand. On
Android the library must be given the application context before its first
HTTPS request (`MobileApplication` → `NativeLibrary.initialisePlatform`,
exported by `core/src/android.rs` under that class's name: rename both
together). The iOS app does not link the library yet.

**iOS builds only on macOS**; elsewhere the two iOS recipes refuse. Android
needs JDK 17, `ANDROID_HOME` (or the SDK in its default place) with the
pinned NDK, the three Android Rust targets and cargo-ndk. The two
apps mirror each other's structure and names: make a change in both. A
version moves in `shared/toolchain.env` and the file `toolchain.sh` names
together. Every user-visible string comes from the string catalogue
(`strings.xml`, `Localizable.xcstrings`), and the app does no networking of
its own: a view model calls the generated API. The reference app id is `io.systemprompt.mobile`.

---

## Debugging & Troubleshooting

```bash
# Quick error check
systemprompt infra logs view --level error --since 1h

# Debug AI request failures
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id>

# Debug MCP tool failures
systemprompt plugins mcp logs <server-name>

# Debug agent issues
systemprompt infra logs trace list --agent <agent-name> --status failed
```

**Key debugging workflow:**
1. `infra logs view --level error` — Find the error
2. `infra logs request list` — Find failed AI requests
3. `infra logs audit <id>` — Get full conversation context
4. `plugins mcp logs <server>` or `logs/mcp-*.log` — Get MCP tool errors

---

## Viewing Governance

Every inference call (`/v1/messages`) and every MCP tool call lands a row in the governance spine. Same CLI surface for both — no separate "gateway logs" vs "tool logs":

```bash
# Every AI request — user, model, token counts, cost, latency, status
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic   # request list filters: --since / --model / --provider (no --status)
systemprompt infra logs trace list --status failed          # only failed runs — --status lives on trace list, not request list

# Full audit for one request — identity, policy evals, prompt, response, cost
systemprompt infra logs audit <request-id>

# Tool-call traces (PreToolUse → decision → spawn → result)
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>

# Cost + usage rollups (hits the same audit table)
systemprompt analytics costs summary
systemprompt analytics requests stats
systemprompt analytics agents
systemprompt analytics tools
```

`logs request list` shows one row per `/v1/messages` hit — the gateway path Pi / any Anthropic-SDK client uses. `logs trace list` shows MCP tool calls. Both are backed by the same 18-column `ai_requests` / trace tables with `user_id`, `tenant_id`, `session_id`, `trace_id` — so `audit <id>` reconstructs the chain from identity to cost.

**`infra logs` vs `analytics` — operational vs dashboard.** The `infra logs request {list,stats}` commands are quick operational views (recent rows, by-provider / by-model aggregate). Their `analytics requests {list,stats}` counterparts are dashboard metrics over a time range with model filtering, cache-hit rate, and CSV export. Same `ai_requests` table underneath — reach for `infra logs` when triaging a live issue, `analytics` when reporting. The `--help` on each cross-references the other.

For live tailing while reproducing an issue: `infra logs view --follow --since 30s`.

---

## Services Configuration

All runtime configuration lives as flat YAML files under `services/`. The root `services/config/config.yaml` is a thin aggregator with an explicit `includes:` list — every resource file must be listed.

```
services/
  config/config.yaml        Root aggregator (includes all resource files)
  agents/<id>.yaml          Flat agent definitions
  mcp/<name>.yaml           Flat MCP server definitions
  skills/<id>.yaml          Flat skill definitions
  skills/<id>.md            Skill instruction bodies (referenced via !include)
  plugins/<name>.yaml       Flat plugin binding descriptors
  ai/config.yaml            AI provider config
  scheduler/config.yaml     Job scheduler
  slack/<name>.yaml         Inbound Slack apps (`slack_apps:` map — ships disabled)
  web/config.yaml           Web frontend config (full WebConfig)
  content/config.yaml       Content source config
```

Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`). Nested `includes:` resolve recursively. Plugin YAMLs are binding descriptors that reference top-level agents, skills, mcp servers, and content sources by id — never inline copies.

---

## Slack (inbound, off by default)

`services/slack/example.yaml` ships **disabled**. Fill in the workspace id, install a
Slack app, put `slack_signing_secret` / `slack_bot_token` in the profile secret store,
then flip `enabled: true`. Core mounts the transport already
(`POST /api/v1/slack/{events,commands,interactivity}`); the binary opts in with the
`slack` feature in `Cargo.toml`.

Route the slash command, not a channel: core's event handler dispatches on both
`message` and `app_mention`, so a routed channel sends every line of chatter to the
agent. The example routes `/systemprompt` to `developer_agent`, which already carries
the `systemprompt` MCP server and `oauth.scopes: [admin]`.

Four gates stand between a Slack message and a tool call, each denying by default:

1. **Workspace** — `authz.allowed_roles` is projected at startup into an
   `access_control_rules` row for `slack_workspace:<workspace_id>` with
   `default_included=false` (`repositories/config/slack_acl.rs`).
2. **Identity** — the sender must map to an account holding the granted role.
   `link_by_workspace_email: true` attaches them to the account owning their *confirmed*
   Slack email; otherwise link by hand with
   `POST /api/public/admin/users/{user_id}/slack-identity` (`{"slack_user_id": "U…"}`),
   `DELETE` to detach. An unlinked sender becomes a role-less first-touch user and fails
   gate 1.
3. **Token** — core mints the A2A token with the sender's own permissions (`admin` role
   ⇒ `Admin`, everyone else ⇒ `User`) and audience `[a2a, mcp]`; an agent declaring
   `oauth.scopes: [admin]` rejects the weaker token.
4. **MCP server** — `services/mcp/systemprompt.yaml` requires audience `mcp` and scope
   `admin`; `rules.yaml` declares who reaches `mcp_server/systemprompt` at all.

Bot scopes: `commands`, `chat:write`, `users:read`, plus `users:read.email` only if
`link_by_workspace_email` is on.

---

## Critical Rules

1. **Core is a crate dependency** — pinned to the published release, in lockstep with this repo's version. Between releases core is pinned by git rev as git dependencies in both manifests (`bridge/CORE_REF`, `just core-pin <sha>`); `main` never is. Adopting a release is `just core-bump X.Y.Z` (every pin, both lockfiles, CORE_REF, local migrate), then `just schema-baseline`, a CHANGELOG entry and `just verify`. Read core's changelog for tightened identifier validators and new `NOT NULL` columns — runtime failures a build cannot catch. See [Branching](#branching-this-repo) and `docs/RELEASING.md`.
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Enterprise Demo`** — Use "Enterprise Demo" for display, "demo.systemprompt.io" for URLs.
6. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".
7. **Demo scripts must work on macOS and Linux** — BSD vs GNU differ on `grep -oP`, `head -n -1`, `sha256sum`, `sed -i`, and binary downloads (pick `hey_darwin_amd64` vs `hey_linux_amd64`). `demo/_common.sh` provides `install_hey()` for the last case; prefer `grep -oE` + `sed -n 's/.../\1/p'` over `grep -oP … \K …`.
8. **No Co-Authored-By in commits** — `coauthorAttribution: false` is set in `.claude/settings.json`. Never add `Co-Authored-By:` trailers to commit messages.

---

## Repository Naming Convention

Every function under `extensions/web/admin/src/repositories/` is named for what
it returns, so a call site reads the same as its signature:

| Returns | Prefix | Example |
|---------|--------|---------|
| `Vec<T>` — zero or more rows | `list_` | `list_top_users` |
| `Option<T>` — a row that may be absent | `find_` | `find_session_header` |
| `T` — exactly one value, or an error | `get_` | `get_request_stats` |
| a page plus its total, `(Vec<T>, i64)` | `list_` | `list_requests_paged` |

Mutations keep the verb that describes them: `insert_`, `update_`, `delete_`,
`set_`, `count_`.

`scripts/check-repository-naming.sh` enforces this: it rejects `fetch_`
outright, and checks every other prefix against the function's actual return
type, so the table above cannot quietly stop being true.

`fetch_` is banned because it is not a synonym for the three above —
it was doing all three jobs at once, which is how the convention drifted: a
reader could not tell from `fetch_summary` whether an absent row was `None` or
an error, and had to open the file to find out.

---

## CSS Files (IMPORTANT)

**All CSS files go in `storage/files/css/`** and must be registered in `extensions/web/src/extension.rs`.

```
storage/files/css/          <- CSS SOURCE (put files here)
extensions/web/src/extension.rs  <- REGISTER here in required_assets()
web/dist/css/               <- OUTPUT (generated, never edit)
```

**To add CSS:**
1. Create file in `storage/files/css/`
2. Register in `extension.rs` `required_assets()`
3. `just publish` to compile templates, bundle CSS/JS, and copy all assets to `web/dist/`

---

## Publishing Assets

After changing templates, CSS, JS, or static files, run:

```bash
just publish
```

This runs (in order): `bundle_admin_css` -> `copy_extension_assets` -> `content_prerender`. Order matters — bundles must be built before `copy_extension_assets` copies them to `web/dist/`. Admin pages are SSR'd at runtime from `.hbs` templates in `storage/files/admin/templates/`, not precompiled.

**Exception: the public-site partials are compiled into the binary.** `services/web/templates/partials/{head-assets,header,footer,scripts}.html` are `include_str!`-embedded by `extensions/web/site/src/partials.rs`. Editing them requires a rebuild (`just build`) and a server restart before `just publish` — running publish alone keeps serving the markup baked into the old binary.

---

## Plugins

Plugins are flat YAML files under `services/plugins/<name>.yaml` that aggregate agents, skills, mcp servers, and content sources by reference:

```yaml
plugins:
  enterprise-demo:
    id: enterprise-demo
    name: "Enterprise Demo"
    version: "2.0.0"
    enabled: true
    agents:
      include: []
    skills:
      include:
        - example_web_search
        - use_dangerous_secret
    mcp_servers: []
    content_sources: []
```

Every id listed must resolve to a real top-level resource in `services/`. `ServicesConfig::validate()` enforces this at load time.
