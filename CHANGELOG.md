# Changelog

All notable changes to this repository are recorded here, newest first.

Conventions (strict — hold every entry to them):

- Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/): an `## [Unreleased]`
  section at the top, then one `## [X.Y.Z] - YYYY-MM-DD` section per release, each with only the
  categories it needs, in this order: `### Breaking`, `### Security`, `### Added`, `- **Code sync** (`/admin/sync`, Platform group): one page for where
  declarations come from and how they move. *Sources* lists the base tree and
  every pinned bundle with its hashes and offers **Import sources**, which runs
  core's in-process services refresh (`POST /api/public/admin/sync/sources/refresh`).
  *Access review* settles the access-control drift one entity per row — apply
  code, keep the database, or export — each with a stated reason, above the
  whole-plane card. *Export & import* downloads every projected plane as one
  zip (`GET …/sync/export.zip`) and stages an uploaded one for a preview at
  `/admin/sync/import/{stage}` that writes nothing until a plane is applied.
  The JSON API behind it is `GET …/sync/status`, `…/sync/planes/{plane}/drift`,
  `…/export`, `POST …/sync/planes/{plane}/apply`, `…/sync/access-control/keep`,
  `…/sync/import`, `…/sync/import/{stage}/apply` and `DELETE …/sync/import/{stage}`;
  reads are open to the console roles, every write needs an administrator, and
  every write and export leaves an activity row (`handlers/sync/`,
  `handlers/ssr/{ssr_sync,ssr_sync_import,sync_plane}/`). The Groups page gains
  a **Sync** tab showing `groups.yaml` against this database with the same
  component.

### Changed`,
  `### Fixed`, `### Removed`. Every breaking bullet leads with `**Breaking:**`, names the
  affected symbol, and ends with `Migrate by …`.
- The heading shape is load-bearing. `just release X.Y.Z` refuses a version with no
  `## [X.Y.Z]` heading, and `scripts/check-release-tag.sh` (a lint gate) requires every version
  heading older than the workspace version to have its `vX.Y.Z` tag — the versioned image only
  exists if the tag was pushed. A version that shipped as a chart but never as an image says so
  in its heading (`## [X.Y.Z] - never released (…)`) and is counted, not checked.
- Entries are written for the reader who did not make the change: full sentences, what changed
  and **why**, named files/commands/flags where the reader will need them. No bare "updated X".
- Every user-visible or operator-visible change lands in `Unreleased` **in the same commit** as
  the change itself. A release moves the `Unreleased` content under its version heading;
  `Unreleased` is never deleted, only emptied.
- Version numbers track the root workspace `version` in `Cargo.toml`, which is core's version
  (lockstep; see `docs/BRANCHING.md`).

## [Unreleased]

### Breaking

- **Breaking:** the per-page CSV URLs (`/admin/requests.csv`,
  `/admin/analytics/cost.csv`, `/admin/governance/warnings.csv`,
  `/admin/governance/secrets.csv`, `/admin/reports/customer.csv`,
  `/admin/reports/internal.csv`) keep answering but are now served by the
  export handler from the matching dataset (`export/legacy.rs`), so their
  columns are that dataset's default columns and headers: money is an exact
  six-decimal `cost_usd`, `warnings.csv` is the decision log alone (safety
  findings are the `governance-findings` table), and the report CSVs still pick
  their table from `?dimension=`. Migrate by reading the new headers, or by
  requesting `/admin/export/{dataset}?format=csv&columns=…` with the columns a
  consumer expects.

### Added

- Console chrome from the upstream admin: the sidebar is now six collapsible
  groups (AI activity, People & access, Governance, Platform, Account,
  Developer). The group holding the current page is always open; the others
  remember the reader's choice (`services/nav-groups.js`). The Access control
  link carries a count of entities where code and the database disagree and no
  one has decided yet (`repositories/sync/attention.rs`), shown to console
  viewers only.
- The header search box suggests matching ids as you type and accepts the
  `short_id` prefixes every list page shows: `GET /admin/api/search/resolve`
  returns `matches` (up to eight, newest first) alongside `url`, and a lone
  match still jumps straight to its page
  (`repositories/governance/suggest.rs`, `services/header-search-list.js`).
- Shared partials for later console pages: `components/icon`, `badge-stack`,
  `sparkline`, `filter-ribbon`, `table-select`, `bulk-bar` and `help-dialog`
  (opened by the page header's `?` through `components/sp-help.js`), with their
  stylesheets.
- A live chart layer for `/admin/analytics`: `components/sp-chart.js` (with
  `sp-chart-draw`, `sp-chart-scale`, `sp-chart-tooltip`) redraws each
  server-rendered `svg-line-chart` at real pixel size with axes, a hover
  crosshair and tooltip, keyboard focus, legend toggles and a table view. The
  server chart still renders complete without scripts. Request volume now plots
  requests, failures and active people on one axis; cost by model and the Cost
  tab's provider-by-day chart are drawn as stacked columns by the same layer.
- An analytics **Skills** tab (`/admin/analytics?tab=skills`): invocations
  (slash vs. tool), people, the conversations that invoked each skill and their
  requests and spend (labelled as conversation spend — rows overlap), and the
  share attributed to a published version, from
  `repositories/analytics/site/skills.rs`.
- One export surface for the console's tables (`extensions/web/admin/src/export/`):
  `GET /admin/export/{dataset}` writes any registered table as CSV, JSON,
  JSON Lines or Markdown with a chosen column set over the window the table's
  contract allows, and `GET /admin/export/{dataset}/preview` answers the row,
  column and cell counts first (50,000-row cap, flagged when it bites). 26
  tables are registered: requests, sessions, traces, conversations by person
  and people, users, groups, projects, "My conversations" and the org-wide
  conversations list, the analytics Models/Skills/Tools/Tool servers/Session
  cost tables, provider cost, cost by day and consumption by container, the
  governance decision log, safety findings and secrets audit, and the five
  month-end report tables. Every page that lists one of them carries an
  **Export** button (`components/export-button`) that downloads CSV without
  JavaScript and opens the export dialog (`components/export-dialog`,
  `services/export*.js`) with it.
- A personal access token is accepted in place of a browser session on
  `GET /admin/export/…` only (`middleware/pat.rs`): it resolves to its owner and
  meets exactly the gates a session does; every other admin route, and any
  write, refuses it. Non-console users may export their own history
  (`/admin/export/history`).
- `util::mcp_tool_name` reduces a host's namespaced MCP tool name
  (`mcp__<server>__<tool>`, `mcp__plugin_<marketplace>_<server>__<tool>`) to the
  server and bare tool the gateway records.
- **Tools & artifacts** (`/admin/tools`, `/admin/artifacts`, Developer group):
  every tool call the platform saw — the model's intent, the execution, the
  governance decision keyed to it and the artifact it produced — read from the
  `tool_activity` view in one statement (`repositories/analysis/tools/page.sql`)
  with KPI tiles, charts, a filter ribbon, a breakdown by tool, server, person,
  client, skill or kind, and a paged, selectable table. The Artifacts page is
  the same page narrowed to calls the one artifact rule says produced something
  a person can view; `/admin/artifacts/{id}` shows one artifact with its
  provenance, scanner findings and stored body, and `…/preview` renders it in a
  sandboxed same-origin frame through core's renderer registry. Four export
  datasets back them (`tools`, `tools-breakdown`, `artifacts`,
  `artifacts-breakdown`).
- **Configuration** (`/admin/configuration`, Platform group): one row per kind
  of configuration under `services/`, projected kinds with their plane's state
  and a link to the owning page's Sync tab, served kinds with the source and
  hash that ship them, plus the `database_cleanup` retention windows read from
  the profile beside what the job last deleted.
- **Observability** (`/admin/system/observability`): the profile's
  `observability.otlp` block beside core's `otlp_export_state` ledger, with
  **Export now** (runs core's `otlp_export_now` out of turn) and **Test
  connection** (posts an empty OTLP/HTTP envelope to the collector); both are
  administrator writes behind the write-origin check. The Code sync page links
  to Configuration and Observability from its header.
- **Data lifecycle** (`/admin/lifecycle`, Platform group): what the retention
  jobs measured, archived and found — the latest size, dead tuples, oldest row
  and week-on-week growth of every managed table, the weekly and monthly
  archives with their SHA-256 and a download link
  (`/admin/lifecycle/archive/{tier}/{period}/{file}`, path segments validated
  against the shapes the jobs write), the last health report's ranked
  findings, and the retention windows in force. Three web-extension jobs
  write it (`extensions/web/jobs/src/retention/`): `retention_daily_report`
  (04:30 daily, into `retention_runs`), `retention_export_weekly` (Sunday
  02:00, the previous ISO week of the raw tables to
  `storage/exports/weekly/`) and `retention_export_monthly` (1st, 02:30, the
  kept rollups to `storage/exports/monthly/`, then the health check and a
  `VACUUM (ANALYZE)`). Every archive is gzipped JSON Lines from
  `COPY … TO STDOUT` with a manifest, restorable with `COPY … FROM`. Runbook:
  `docs/ops/retention-and-backups.md`.
- **Connectors** (`/admin/connectors`, Account group): a person's connections
  as cards grouped by what to do next — needs attention, ready to connect,
  connected, nothing to do — under a health strip that names the next step.
  **Test connection** now returns a step-by-step report (credential, MCP
  session, tools, identity) with each stage's outcome and duration
  (`services/connector_oauth/report.rs`), shown inline on the card.
  Session-attested servers — `oauth.required` with scopes and no `connector:`
  block, such as the `systemprompt` control plane — appear as connectors that
  are live for anyone whose roles carry the scopes, tested by a live check
  (`services/connector_readiness.rs`). Generic connectors gain
  `connector.display_name`, `authorization_params` and `identity: userinfo`
  (read `sub`/`email` from the issuer's OIDC userinfo endpoint), and issuer
  identifiers are compared as URLs with a single trailing slash ignored.
- **Connect a client** (`/admin/connect`, Account group): the three-step
  connect-code wizard for Claude Code, Claude Desktop and OpenCode, the bridge
  downloads and the client guides, on a page of its own.

### Changed

- Avatars take a stable per-person tone from the new `avatar_tone` helper and
  one size scale (`05-avatar.css`, `01-tokens-avatar.css`) instead of a single
  gradient. Expandable table rows (Devices, By person) are driven by the shared
  `services/table-expand.js`: the whole row is the trigger and the detail
  slides open, replacing the per-page toggles.
- The model-usage chart is a donut drawn as SVG arcs with a per-slice tooltip
  and the request total in its centre, replacing the CSS conic-gradient disc;
  its legend rows carry a share bar. KPI tiles accept an `icon` and a
  sparkline `spark` in a reserved trend row.
- `util::time_range` gains a `90d` preset and a public `TimeRangePreset::parse`,
  `duration`, `as_str` and `label`; the sessions and traces pages read the
  preset name from it instead of their own tables.
- The header actions and install menus now bind to the ids the templates
  actually write (`header-actions`, `install-menu`); previously neither control
  was wired and the install button did nothing at narrow widths.
- The admin SSR router now takes the shared `DbPool` and builds core's
  managed-resource repository and OTLP export ledger once
  (`routes/managed_state.rs`); `admin_ssr_router` returns
  `Result<Router, StateError>`. The six legacy CSV URLs moved into
  `routes/ssr_export.rs` and the governance pages into
  `routes/ssr_governance.rs` (same paths). `handlers::ssr::format::relative_time`
  now delegates to `systemprompt_web_shared::format::relative_time`, which
  gains `truncate_chars`, `truncate_ellipsis` and `compact_num`.
- `plugin_usage_retention` now calls `expire_raw_evidence` (schema
  `32_raw_retention.sql`): hook events, gateway requests and what hangs off
  them expire together after 90 days, while `conversation_facts`,
  `conversation_skill_facts` and the daily rollups are kept. It and the three
  `retention_*` jobs now have explicit entries in
  `services/scheduler/config.yaml`.
- The profile page no longer carries the connect wizard or the connectors
  list; its header links to **Connect a client** and **Connectors** instead,
  and a finished OAuth consent returns to `/admin/connectors#connector-<id>`.
  `profile-connect-code.js`, `profile-connections.js` and `05-connection.css`
  are replaced by `connect-code.js`, `connectors.js`,
  `services/connector-labels.js` and the `20-page-connect*` /
  `20-page-connectors*` stylesheets.
- Connector token handling: every 4xx from a token endpoint now retires the
  grant as a reconnect (RFC 6749 §5.2), where only `invalid_grant` or a 401
  did before; a 5xx, 429 or transport failure stays an outage and keeps the
  grant. A broker call within 30 seconds of a recorded outage is held without
  another provider call; Test connection always goes through. A server the
  access rules admit is withheld from the bridge manifest until the person's
  connection to it is ready, and the manifest's diagnostics say why.

### Removed

- The hand-rolled CSV builders (`handlers/ssr/csv.rs`, `governance/csv_export.rs`,
  `ssr_analytics_dashboard/csv.rs`, `ssr_report_customer`, `ssr_report_internal`
  and the requests/secrets CSV handlers), replaced by the export layer.
- The server-only stacked chart (`types/svg_stack.rs`,
  `components/svg-stacked-chart`): both of its charts are now drawn as columns
  by the live layer from the ordinary line-chart view.
- `73-analysis.css`: every rule in it styled an analysis page this branch does not render; the analysis suite brings its own stylesheet back.

## [0.62.0] - 2026-09-28

### Breaking

- **Breaking:** core 0.62.0. `secrets.json` must carry `encryption_master_key` (32 bytes as 64 hex characters): core refuses to boot without it, before migrations run, where 0.61 died later in extension init. `admin setup` mints it from 0.62.0 on, the Docker image mints it once into a persisted profile that predates it (`docker/container-state.py`), and `just setup-local` adds it to an existing local profile. Migrate by adding `openssl rand -hex 32` as `encryption_master_key` to every operator-managed secrets file (helm `profile.existingSecret`, the air-gap and scaled profiles) before upgrading; a supplied `SYSTEMPROMPT_PROFILE_DIR` without it now fails at the entrypoint with that instruction.

### Security

- The template owns its starter `secret_scan.patterns` catalog, including separate AWS access-key id and secret-value rules. Core supplies the validated scanner and recovery engine without activating vendor signatures. The response scanner consumes the same compiled catalog as ingress governance.

### Added

- **Tooling:** `scripts/sync-core-version.sh` (every core crate pin in both workspaces plus `bridge/CORE_REF`), `scripts/check-core-ref.sh`, `scripts/check-core-crate-versions.sh`, and the recipes `core-pin`, `core-guard` (`just deploy` runs it), `core-checkout`, `schema-baseline`, `fetch-release`, `stop`, `verify`, `preflight*`, `doc-check`, `init-hooks` (tracked `.githooks/pre-commit`), `hack`, `lint-silent-skips`, `lint-no-untyped-admin` and `coverage*`.
- **Gates:** `check-discarded-results`, `check-fail-open` (core's rust-contracts scanner), `check-migration-numbers`, `lint-layers`, `lint-repo-construction`, `check-json-value`, `lint-silent-skips`, `check-field-copy-from`, `check-dockerfile-paths`, `check-dropped-schema`, `check-core-ref`, `coverage-badge` and `check-docs-version` join `just lint-gates`. `check-schema-baseline` is the release ladder (floor 0.61.0; rungs 0.61.0 and 0.62.0) and is in the array. `tests/integration/schema-upgrade` restores every rung, seeds 2000 rows per hot table (`seed_hot_tables.sql`), upgrades it and requires the rows to survive and the shape to equal a fresh install; the single `release-baseline.sql` is retired.
- **Gates:** `tests/unit/web/src/migration_cost.rs` (core's `audit_one` detector over this repo's migrations, with `plugin_usage_events`, `conversation_facts`, `mcp_tool_executions` and `governance_decisions` added to core's hot-table list): a migration that rewrites a hot table must lead with `-- @cost: rows=… measured=… triggers=…`, and a new `FOR EACH ROW` trigger may not fan out beyond its row. 068, 071 and 073 are grandfathered; the unreleased 087 and 091 carry estimated directives.
- **Tests:** `tests/integration/gateway` (provider abstraction, shadow-AI, quota config, residency and no-retain requirements, run by `just test-integration`); `tests/unit/web/src/services_tree_declarations.rs` (every id the services tree declares reaches its projection); `tests/common` sweeps orphaned throwaway databases a killed run left behind (`orphans.rs`) before it creates one.
- **Coverage:** `scripts/coverage.sh` / `coverage-check.sh` (floor and ratchet against `coverage/baseline.json`) and a nightly `coverage.yml`; no baseline is recorded yet.
- **Schema:** declarative schemas and one migration each (slots 083–094) for `sync_state`, `service_sources`/`service_owned_ids`, `marketplace_versions` (+ `marketplace_version_at`), `conversation_analyses`, `ai_request_scopes` (stamped by an `AFTER INSERT` statement trigger on `ai_requests`; migration 087 backfills history from each person's current primaries), validity windows on `group_members`/`project_members`/`user_manual_roles` plus `access_control_rule_validity`/`user_device_cert_validity`, `gateway_routes`/`governance_chain`, `tool_activity` (+ `artifact_kind`) and `analysis_reports`, `conversation_facts`/`conversation_skill_facts` and their refresh functions, the `user_last_seen` view, the retention ledger, and `expire_raw_evidence`. The declarative files are registered in `extensions/web/src/schemas.rs` (46 before 45: the facts refresh reads the `tool_activity` view), so a fresh install has every table an upgraded one gets from the migrations; the pages and jobs that read them arrive with the admin-console port.
- **Access control (runtime):** boot reads `services/access-control/rules.yaml` through the new sync data layer (`repositories/sync/*`, planes `access_control`, `groups`, `gateway_policies`, `gateway_routes`, `governance`): a plane whose projection is empty is seeded from its file, every other boot compares and logs the drift (`sync_drift`) and writes nothing. `repositories/access_control/*` projects, diffs, reviews, exports and applies the declaration; `authz/connector.rs` adds the `connector` band (precedence 160); rule validity windows bind through the new hourly `access_expiry` job, and calendar-month quota windows through `quota_month_window`. Slack apps still project `slack_workspace:<id>` from `services/slack/*.yaml`, and those rows are excluded from the access-control drift. The console pages for sync and review arrive with the admin-console port.
- **Access control:** `services/access-control/rules.yaml` is the one declarative source of entitlement — one entry per entity with a required `why`, `default: open|closed`, optional `valid_until` and `owner: bundle:<name>`, and allow/deny bands (`role`, `group`, `project`, `connector`). `scripts/validate-services.sh` checks it (entities exist, groups/projects are declared, no marketplace carries an `access:` block, plugin and marketplace includes resolve, marketplace JSON is not stale). New docs: `/documentation/access-control` and `/documentation/services-sync`.
- **Kits:** `deploy/kit/` is the template for a kit repository that publishes a signed services bundle (publish and stats workflows, `tools/sanitize-kit.py`, runbook); `deploy/kit/known-kits.json` lists none yet. `just services-pin <kit> <digest|channel> [profile]` writes the profile's `services.sources[]` entry. `just kit-export <marketplace>` runs the new `extensions/cli/kit-export` crate (`systemprompt-kit-export`), which exports a marketplace from `services/` into a kit repository tree in Anthropic marketplace format and verifies the round trip; sidecars also carry agents, artifacts and content sources. New docs: `docs/kits-on-another-instance.md` and `docs/CONFIGURED-CONNECTORS.md`.
- **Scheduler:** core jobs `managed_inventory_refresh` (also run at boot), `oauth_cleanup`, `user_rate_limit_prune`, `thought_signature_cleanup` and `otlp_export` are scheduled.
- **Models:** `claude-opus-5-5` in the provider catalog ($4 / $20 per million input/output tokens).

### Changed

- **Core:** every core crate pin (both workspaces, `extensions/web`) is 0.62.0, `bridge/CORE_REF` is `v0.62.0`, and the workspace version follows it (lockstep).
- **Docker entrypoint:** migrations run as `infra db migrate --repair-drift` (repairs checksum drift only, then retries once) instead of a blind `migrate-repair --apply` and retry; `DATABASE_WRITE_URL` lands in the secrets as `database_write_url`; `SYSTEMPROMPT_ADMIN_EMAIL` is accepted beside `ADMIN_EMAIL` and wins when both are set; every node runs `publish_pipeline` before serving, because the scheduler's boot run takes a database-wide lock and renders only one node's `web/dist`. A supplied `SYSTEMPROMPT_PROFILE_DIR` (helm, air-gap; mounted read-only) takes `DATABASE_URL` from its secrets when unset and must already carry `encryption_master_key` and either `signing_key_pem` or the key file its profile names — nothing is minted into a shared profile.
- **Release:** `just release X.Y.Z` replaces `just gate` / `just promote` and the mutable `promote` ref. It requires a green `Gates passed` on the exact `next` push commit, freezes it on `promote/X.Y.Z/<main>/<sha>`, opens the PR onto `main`, and — run again once the PR's `Verify frozen promotion` proof is green — merges it and pushes the `vX.Y.Z` tag at the merge. `release-gateway.yml` re-verifies that merge (`scripts/check-release-merge.sh`) before building anything. See `docs/RELEASING.md` and the new `docs/BRANCHING.md`.
- **CI:** `.github/workflows/gates.yml` replaces `ci.yml` and `quality.yml` and now runs on every push to `next`: independent static, lint, test (Postgres 18), e2e (Playwright) and audit/deny/machete tiers plus a `Gates passed` aggregate for the `main` ruleset. `just verify` runs the same static, lint and test tiers locally.
- **Images:** a release image gets `:X.Y.Z`, `:X.Y`, `:X` and `:latest` only after the per-arch smoke and a new `upgrade-boot` job (the image booted over every recorded release schema with 2000 seeded rows per hot table) pass; until then it carries only `:sha-<7>`. Probes now require a finished-boot `/health` body, not any 200.
- **Build:** `scripts/build-coordinator.sh` has no success cache (every run compiles the latest tree; `BUILD_FORCE` is a no-op) and refuses to start a compile when the volume holding `target/` has less than `BUILD_MIN_FREE_GB` (default 25) GB free.
- **Versions:** `scripts/sync-release-version.sh --check` enforces lockstep (the release's `MAJOR.MINOR` equals core's) and `sync-release-version.sh` now also rewrites the version literals in `docs/install/*.md` and `deploy/*/*.md`, which `scripts/check-docs-version.sh` keeps on the workspace version.
- **Dockerfile:** the pinned toolchain is its own cached layer, an `artifacts` stage exports the binaries, and `/app/storage/data` exists owned by `app`, so a named volume mounted there is writable.
- **Breaking:** **Access control:** `services/access-control/roles.yaml` and `departments.yaml` are deleted and no longer read; the `enterprise-demo` marketplace carries no `access:` block and every grant lives in `rules.yaml`. Boot no longer upserts the access tables on every start: it seeds an empty table once and afterwards only reports drift, so a console edit survives a restart. Migrate by moving any local `roles.yaml`/`departments.yaml` grant into a `rules.yaml` entity with a `why`, then reconcile with the database (the drift is in the boot log until the sync page lands).
- **Gateway:** routes carry a `name` and `description`; `default_model` is `claude-sonnet-5[1m]` so Claude Code budgets the model's 1M context; `quota_fault_mode: closed` refuses a request whose quota subject cannot be resolved instead of passing it uncounted. `services/gateway/policies.yaml` pins `history: off` and the full heuristic phrase list and documents the warn-mode semantics; thresholds are unchanged.
- **Governance:** `services/governance/config.yaml` carries its doctrine header (warn mode as a measurement window; inherited `mode`); the pattern list is unchanged.
- **Schema lint:** `scripts/lint-schema.sh` ignores statements inside dollar-quoted function bodies and exempts `schema/retire/`.

### Fixed

- `scripts/validate-release.sh` refused every release since the core-0.61 migration because it counted the inert `[workspace.metadata.unreleased-core-patch]` table as a live core patch.
- The operator docs named 0.2.2 and 0.49.0 and non-existent tarball names; they now name the current version and `systemprompt-gateway-<version>-<target>.tar.gz`.
- The access-control page's open-entity count swallowed a database error silently; it is logged now.
- Every managed MCP server in `services/mcp/*.yaml` now declares `tool_policy: allow`. Core 0.53.0 makes the key mandatory: a server without it is withheld from the signed bridge manifest and rejected by boot validation, so the deployment would have refused to start. `allow` keeps today's effective behaviour.
- The admin sidebar fell back to another deployment's domain for its logo alt text and label when `branding.domain` is unset; it now falls back to `systemprompt.io`.

### Removed

- **Breaking:** Salesforce identity and org support, which served another deployment: the Salesforce SSO/identity link (`/admin/users/{id}/salesforce-identity`, `/admin/api/profile/salesforce/unlink`), the per-user bearer accessor (`/api/public/salesforce/token`), JWT-bearer org provisioning, the Salesforce connector-OAuth provider and its user-detail row, and schema `21_salesforce_identity.sql`. Migration 095 drops `salesforce_user_identities` on established databases. The generic connector-OAuth machinery stays, and `/connectors/{provider}/reprovision` now works for any configured provider (its error code is `provider_reprovisioned`). Migrate by removing any `access-control/salesforce.yaml` and Salesforce MCP/connector config before upgrading; ADFS SSO is unaffected.
- The orphaned evals admin templates (`evals.hbs`, `eval-run-detail.hbs`, `partials/evals/*`) and `css/admin/20-page-evals.css`: core 0.61 retired evals and nothing renders them. The `/admin/evals` contract variants and the never-compiled `evals_repositories.rs` test go with them.
- `tests/unit/web/src/india_skills.rs`, a test for another deployment's skill inventory.

## [0.61.0] - never released (included in 0.62.0)

The workspace moved from 0.49.0 to 0.52.0 (2026-09-14) and to 0.61.0 (2026-09-25) without a tag or image; its changes ship in 0.62.0.

### Breaking

- **Breaking:** core 0.61.0. Evaluations are retired in core: the evaluation scheduler job, the eval session attribution and the evals admin routes are gone. Departments are retired from the admin console and E2E surface. Migrate by removing any `scheduler.jobs` entry that names an evaluation job and any `departments` reference outside `services/access-control/`.

### Security

- **Core:** the rustls dependency is pinned to its security repair (2026-09-14).

### Added

- **Evaluation (0.52.0):** the managed optimization pipeline was backported, then retired again with core 0.61.
- **Governance (0.52.0):** configured secret-scanning work is preserved across restarts.

### Changed

- **Core:** migrated to core 0.61 — the extension graph links core's migration extensions into every test harness, fresh and upgraded schemas converge, the web billing model is retained under core's ownership rules, governance rows and layout follow core's template, and `just setup-local` generates the gateway `encryption_master_key` locally.
- **Tests:** the admin contract suite and Playwright E2E follow the core 0.61 route set; retired helpers, routes and department coverage were removed.

## [0.49.0] - 2026-09-09

### Security

- An entitlement lookup that fails now denies the request instead of resolving
  as empty. The four subject-attribute providers behind the access matrix —
  department, group, project and Salesforce — used to swallow a database error
  into "this user holds no values for that dimension". Every deny rule keyed on
  that dimension then failed to match, and the request was allowed on the
  strength of the error. Core 0.49.0 makes
  `SubjectAttributeProvider::values_for` fallible to remove exactly that, and
  the callers propagate: the governance authz webhook answers 200 with an
  explicit deny, and the gateway catalogue, marketplace filter,
  effective-permissions view and access matrix surface the error rather than
  rendering a permissive view of it.

### Changed

- Adopt published systemprompt core 0.49.0 across both workspaces, the Helm
  chart, the CasaOS and DigitalOcean deployment artifacts and `bridge/CORE_REF`;
  both lockfiles are re-resolved.
- Correct the demo index: 45 category scripts, two of which make live model
  calls (`governance/09-pi-agent.sh` and `governance/10-safety-scanner.sh`).
  Both counts and the free/paid split were a release behind.

## [0.48.0] - 2026-09-08

### Changed

- Align both workspaces and release artifacts with published core 0.48.0.
- Use persistent paid Render hosting and document Railway template configuration.
- Preserve container profiles, signing identity and uploaded files across redeploys.

### Security

- Public registration cannot grant administrator roles and respects registration policy. Container deployments disable self-registration by default; administrators issue passkey setup links through the CLI.


## [0.47.0] - never released (included in 0.48.0)

Tracks systemprompt-core **0.47.0**. The workspace moves from 0.42.1, so this
entry also covers core 0.43.0 through 0.46.0: the template was out of scope
for those releases, no `v0.43.0`–`v0.46.0` tag or image was ever published,
and there are no entries missing from this file. The 0.44.0 configuration move
below was written for a release that never shipped and lands here instead.

### Release reliability

- Builds and Clippy use locked dependencies and the offline SQLx cache, without updating dependencies or migrating an operator database during compilation.
- Fix marketplace authorization test fixtures for core 0.47.0 and the proc-macro-error2 compiler compatibility warning.
- Release archives and images are built from the validated release commit and include runtime templates, static assets, and MCP manifests.
- Require candidate container boot, exact-version multi-architecture smoke tests, upgrade/restart persistence, and Helm installation before chart publication.
- Fix first-boot admin email in browser CI, native installer signature identity and resource installation, and cleanup credential diagnostics.

### Admin console redesign

The admin console is rebuilt on the `sp-` design system backported from the
astound fork: OKLCH tokens, a filename-ordered CSS cascade under
`storage/files/css/admin/`, namespaced component partials (breadcrumbs,
page-header, section, toolbar, table, sort-header, pagination, tabs, kpi,
badge, notice, empty-state, time-range, charts) and typed page view models
(`BreadcrumbView`, `SortHeaderView`, `TabLinkView`, `Pagination`,
`TimeRangeContext` with a rejected-bounds flag). Every page now renders one
breadcrumb trail, one header, at most one KPI band, one toolbar and one-line
table rows; the density bar is rows ≤36px, KPI band ≤110px, no horizontal
scroll.

- **Breaking:** admin routes are flat. `/admin/access/users` → `/admin/users`,
  `/admin/access/departments` → `/admin/departments`, `/admin/access/tokens`
  → `/admin/access-tokens`, `/admin/access/matrix` → `/admin/access-control`,
  `/admin/governance/policies` → `/admin/governance`, and
  `/admin/entities/{requests,sessions,traces,contexts}` →
  `/admin/{requests,sessions,traces,contexts}`. Every old path answers `308`
  to its new home for one release (`routes/ssr_redirects.rs`); `/admin` lands
  on Evals.
- The sidebar is five groups in operator order: People & access, AI activity,
  Governance, Platform, Account. The dark-mode toggle is retired with the old
  stylesheet; the design system is light-only.
- Every page is converted, template-only ones included: users and user detail
  (three tabs), departments, access tokens, access control, requests and the
  request audit trail, sessions, traces, contexts and their detail pages,
  evals and run detail, governance policies / decisions / hooks / policy edit,
  the trace demo, models, profile, settings, setup, and the shell-less login,
  register, passkey and verify pages.
- New gates: every admin template must register with the Handlebars engine
  (`template_parse`), every field a template reads must be defined somewhere
  (`scripts/check-template-fields.sh`), helper-name shadowing is refused
  (`template_helper_names`), and the admin CSS-class and front-end-standards
  gates moved from `extensions/web/tests/` to the `tests/` workspace. The
  fork-drift gate is gone: the template no longer tracks a sibling tree.
- A Playwright suite under `playwright/` walks every admin page as admin, user
  and anonymous with a four-block spec per page and a measured density bar;
  `just e2e-install`, `just e2e-seed`, `just e2e`, `just e2e-gate`, and an
  `e2e` job in CI.
- Core 0.47.0: `Decision::Warn` (warn mode) is handled everywhere the chain's
  decision is matched — audited as `warn`, returned to the caller as allow.

### 0.44.0 configuration move (never shipped on its own)

The headline is a configuration move. Core 0.44 reads the provider catalog and
the gateway routes from the services tree instead of the profile, and a profile
that still carries either key does not boot. **Every existing deployment needs
the migration below before it will start on this image.**

### Breaking

- **Breaking:** `providers:` and `gateway:` are no longer profile sections. Core
  0.44 fails boot with `ProfileError::MovedToServices` on a profile carrying
  either key, naming the key and the file it belongs in. The catalog now ships
  with the image alongside the agent and MCP trees, so every environment boots
  the same models and pricing and only the credentials named by `api_key_secret`
  differ. Migrate by moving the `providers:` block from
  `.systemprompt/profiles/<name>/profile.yaml` into `services/ai/providers.yaml`
  and the `gateway:` block into `services/ai/gateway.yaml`, each keeping its
  top-level key, then adding both to `includes:` in `services/config/config.yaml`
  as `../ai/providers.yaml` and `../ai/gateway.yaml` — with the `../`, because an
  include resolves relative to the directory of the file that lists it, and that
  file is `services/config/config.yaml`.

- **Breaking (from core):** `Config` gained `metrics_port`. `/metrics` is no
  longer mounted on the public router. Migrate by setting `server.metrics_port`
  in the profile to expose metrics on their own listener; code constructing
  `Config` literally must set the field, and `None` preserves current behaviour.

### Added

- `deploy/scenarios/airgap/services-ai/` — the air-gap scenario's own catalog,
  bind-mounted over `/app/services/ai`. The shipped catalog names public
  provider endpoints that the sealed `internal: true` network has no route to,
  and providers merge by concatenation with duplicate-name-is-an-error, so one
  file cannot carry both a real and a mock `anthropic`. Every model resolves to
  the in-network mock-inference container and both route ids are unchanged, so
  `01-egress-assert.sh` proves what it proved.

### Changed

- The provider catalog ships as `services/ai/providers.yaml` (3 providers, 45
  models) and the routes as `services/ai/gateway.yaml`. Distinct from
  `services/ai/config.yaml`, which remains the AI domain's own configuration —
  per-provider toggles and defaults, not a priced catalog.

- The `scaled` scenario now serves the full shipped model list. Its previous
  inline catalog was a strict subset of local's — the same three providers and
  endpoints with fewer models — and a single shipped catalog is 0.44's stated
  intent, so the extra models are exposed there too. Nothing errors on upgrade;
  the surface simply widens.

- The admin gateway editor writes `services/ai/gateway.yaml` and never the
  profile. A route edited through the UI previously wrote a `gateway:` key back
  into `profile.yaml`, producing a profile core 0.44 refuses to boot.

- `docs/profile.schema.json` is regenerated against the 0.44 profile type. It
  had been stale since 0.42.0: `SecurityConfig.login_page_url` and
  `SystemAdminConfig.email` are 0.43 additions it never carried, and only
  `ServerConfig.metrics_port` comes from this release.

- `docs/RELEASING.md` Step A described committing directly to `main`. `main` is
  release-only and reached by pull request; the step now ends on `next` and names
  the `just gate` / `just promote` / merge sequence the repository already uses.

### Fixed

- **Security:** the admin gateway catalog is read from the loaded services tree,
  and an absent catalog is an error rather than an empty list. The repository
  read `gateway:` from the profile and ended in `.unwrap_or_default()`, so
  against a migrated profile it returned zero routes with no error at all: the
  per-user gateway catalog was empty for every user, the admin pages showed no
  routes, and the after-the-fact ACL detector — which iterates that same list —
  reported no violations while checking nothing. A governance surface may not
  fail open and quiet.

- **Security:** the access-control entity catalogue lists gateway routes again.
  **This was broken before the 0.44 migration, not by it.**
  `build_gateway_routes` guessed at a profile path, trying
  `<services>/../profile.yaml` and then a hardcoded
  `<services>/../.systemprompt/profiles/local/profile.yaml`, and fell through to
  an empty list. `.systemprompt/profiles/` is gitignored, so in any deployed
  image neither candidate can exist, and the catalogue rendered "No entities of
  this type configured" for gateway routes regardless of configuration. It reads
  the loaded services tree now, and an unreadable catalog is logged rather than
  rendering as an absence of routes.

## [0.42.1] - 2026-08-31

Tracks systemprompt-core **0.42.0** — a template-only patch. No core change.

### Fixed

- Container deployments could not complete first boot. `docker/entrypoint.sh`
  called `admin setup` without `--admin-email`, which core has required since
  0.41.0, so every image-based install died with "An administrator email is
  required" before serving anything. It affected every channel that boots this
  image — compose, Coolify, Dokploy, Portainer, CapRover, CasaOS, Railway,
  Render, Zeabur, Northflank and the DigitalOcean 1-Click droplet. The binary,
  Homebrew, Helm and crates paths of 0.42.0 were unaffected.

  `ADMIN_EMAIL` is now required rather than defaulted. Core stopped inventing an
  address deliberately: the fabricated one was displayed as the operator's
  identity on the device-link consent screen, directly above the control that
  mints a durable personal access token. The entrypoint names the variable and
  explains it, the compose templates declare `${ADMIN_EMAIL:?…}` so the failure
  arrives at `docker compose up` with a usable message rather than in a crash
  loop, and both compose smokes now set it.

## [0.42.0] - 2026-08-31

Tracks systemprompt-core **0.42.0**. This entry also carries the work written
up as 0.41.0, which was never released: the template was out of scope for that
core release, so no `v0.41.0` tag and no `0.41.0` image were ever published.
Recording it as shipped would have advertised a chart appVersion whose image
does not exist, which is the failure `check-release-tag.sh` exists to prevent.

### Changed

- **Breaking (from core):** `DenyReason::HookUnavailable` gained a `detail`
  field. The governance webhook's `deny_for_auth_failure` had been smuggling
  the cause into `policy` as `auth_failure: <reason>`, which made every distinct
  failure its own policy name and nothing groupable; the cause now goes in
  `detail` and `policy` is the constant `auth_failure`.

### Fixed

- **Security:** the governance hook's `agent_id` can no longer raise the
  caller's scope. `POST /hooks/govern` looked the payload's `agent_id` up in
  `services/agents/*.yaml` and took the higher of that scope and the caller's
  own, so a user-scoped token naming an admin-scoped agent was governed as
  admin — waiving the tool blocklist and the approval hold. The value is a
  self-report (a Claude Code subagent id) and never a platform agent. Scope now
  derives from the token's permissions and the user's stored roles only; the
  reported id is kept in `evaluated_rules` under `principal.claimed` and the
  `agent_id` column holds credential-derived identity alone.
  `resolve_agent_scope` and `load_all_agent_scopes` are removed outright.
- `bridge/CORE_REF` is tracked. Sixteen workflow steps read it to materialise
  the core sibling checkout, and on a clean CI checkout it did not exist.
- Three stale intra-doc links to `systemprompt_security::authz::resolve`, which
  moved to `authz::resolver::` in core 0.41.0.

### Changed

- **Breaking:** `rate_limits.tier_multipliers` is gone from the profile schema,
  following its removal in core 0.41.0. `RateLimitsConfig` is
  `deny_unknown_fields`, so a profile still carrying the block fails to load —
  delete it.
- The admin extension follows core's typed entity catalog and its three-way
  `Decision`: a rule may now resolve to `Pending`, which the hook answers as
  "ask" rather than collapsing to allow or deny.
- The minimum supported Rust version is 1.96, and the MSRV job now actually
  tests it.
- CI gates pushes to `next`, not only pull requests.
- `analytics`: `count_concurrent_sessions` and the actions-per-minute metrics
  are removed.

## [0.40.0] - 2026-08-26

Tracks systemprompt-core **0.40.0**. Release-process work: promotion freezes a
ref so a concurrent push cannot ride into `main`, `next` becomes the default
branch, and the scheduled gate is dropped in favour of an on-demand pre-release
cycle. Two call sites realigned with core.

## [0.39.0] - 2026-08-25

Tracks systemprompt-core **0.39.0**. The trace-list query is cached and four
stale fixtures refreshed; setup-phase guides point at documentation pages that
exist.

## [0.38.0] - 2026-08-25

Tracks systemprompt-core **0.38.0**. Carried no template-side changes of its
own — released to keep the template's core pin current.

## [0.37.1] - 2026-08-25

Tracks systemprompt-core **0.37.0** — the first template release whose version
does not match the core release it carries. Helm chart 0.19.1 with appVersion
0.37.1; the CasaOS, DigitalOcean, and Packer manifests pin the 0.37.1 image.

### Fixed

- A trace written by an enforcement site that made no AI request could not be
  resolved. The lookup searched `ai_requests` alone, but such a decision only
  ever writes a `governance_decisions` row, and since core 0.34.0 that row
  carries its own `trace_id` — so those traces resolved to nothing and the
  governance chain behind them was unreachable. The lookup now unions both
  tables.
- The trace list and stats no longer read a `trace_id` as if it were a
  `session_id`. That fallback existed to keep governance-only rows visible; it
  conflated two identifiers, and those rows now surface through the resolver
  above instead.

### Changed

- `scripts/sync-release-version.sh` accepts `CORE_VERSION`, so a template
  release can name a core release with a different number. Without it the script
  pinned the core crates to the template's own version, which for this release
  would have named a core 0.37.1 that was never published. In `--check` mode it
  defaults to the pin already in `Cargo.toml`, so the release guard asserts that
  every core pin agrees rather than that it equals the tag. The chart bump now
  follows the shape of the release: a patch release bumps the chart's patch.

## [0.37.0] - 2026-08-24

Tracks systemprompt-core 0.37.0. Helm chart 0.19.0 with appVersion 0.37.0; the
CasaOS, DigitalOcean, and Packer manifests pin the 0.37.0 image.

### Changed

- The scaled scenario runs the scheduler on **every** replica rather than a
  single dedicated container. A replica now claims each job with a Postgres
  advisory lock keyed on the job name (`scheduler.distributed_lock`, on by
  default) and the losers skip, so the dedicated scheduler node and the
  `scheduler-disabled` config override — both deployment-time mitigations for a
  limitation the engine has since fixed — are gone. The two Kubernetes
  Deployments collapse into one, and `04-scheduler-isolation.sh` becomes
  `04-scheduler-exactly-once.sh`: it proves every replica starts the engine and
  the job still executes exactly once, which is the stronger property and the
  one that survives losing a node.

### Fixed

- Three pieces of configuration were read from process-global state, so tests
  that varied them were correct only one-per-process. The MCP CLI's binary and
  working directory now come from a `CliLocation` resolved at the composition
  root, the ingestion job takes `delete_orphans` as a job parameter and resolves
  its blog config from the job context's own `AppPaths`, and the
  subject-dimension registry is cached per database.

## [0.36.0] - 2026-08-24

Tracks systemprompt-core 0.36.0. Helm chart 0.18.0 with appVersion 0.36.0; the
CasaOS, DigitalOcean, and Packer manifests pin the 0.36.0 image.

### Fixed

- **Three pieces of configuration were read from process-global state**, so the
  tests that varied them were only correct one-per-process and `cargo test`
  produced failures that were not real. The MCP CLI's binary and working
  directory now come from a `CliLocation` resolved at the composition root
  (replacing `SYSTEMPROMPT_CLI_PATH`/`SYSTEMPROMPT_WORKDIR`); the ingestion job
  takes `delete_orphans` as a job parameter and resolves its blog config from the
  job context's own `AppPaths` rather than the process-wide
  `BlogConfigValidated::cached()`; and the subject-dimension registry is cached
  per database instead of in a single `OnceLock` bound to whichever pool asked
  first. None of those environment variables was sanctioned, and nothing outside
  the tests ever set them. The suite passes under `cargo test` as well as
  `cargo nextest`: 828 tests, no failures.

### Added

- `services/slack/example.yaml` documents `link_by_workspace_email` again. Core
  0.36.0 implements it: `SlackClient::user_info` reads the sender's profile and an
  address Slack reports as confirmed attaches them to the account that already
  owns it. It was removed in 0.35.0 because the field existed only in this example
  and made a clean `setup-local` fail; the shipped-YAML gate added then keeps that
  from recurring.

## [0.35.0] — chart only, never released

Chart 0.17.0 went out with `appVersion: 0.35.0`, but no `v0.35.0` tag was ever
pushed, so `release-gateway.yml` never ran and
`ghcr.io/systempromptio/systemprompt-template:0.35.0` does not exist. There is no
installable 0.35.0: everything listed below reached users in 0.36.0. The entry is
kept rather than deleted because the chart version is public.

The 0.30.0 through 0.34.0 template releases were cut without changelog entries;
this entry covers only the work landed since 0.29.0 that had not yet been written
up, and the gap above it is acknowledged rather than reconstructed.

### Added

- **Inbound Slack, shipped disabled.** `services/slack/example.yaml` declares an app (workspace
  id, signing secret and bot token by reference, `enabled: false`) and routes the
  `/systemprompt` slash command to `developer_agent`, which already carries the `systemprompt`
  MCP server and `oauth.scopes: [admin]`. Route the command rather than a channel: core
  dispatches on every `message` in a routed channel, not only `app_mention`. The binary opts
  into the transport with the `slack` feature in `Cargo.toml`.
- **`POST|DELETE /api/public/admin/users/{user_id}/slack-identity`** — link or detach the Slack
  account a user drives the platform from, for accounts whose Slack profile carries an
  unconfirmed or different email and so cannot be linked automatically. Body:
  `{"slack_user_id": "U…"}`. Writes a `federated_identities` row under `https://slack.com`
  (`repositories::users::federated`) and refuses to steal a mapping owned by another user.

### Changed

- **A Slack app's `authz.allowed_roles` is now enforced, not just documented.** Core wrote the
  projection (`ingest_slack_apps`) but nothing called it, so the field described an intention no
  rule backed. `repositories/config/acl_yaml_loader.rs` runs it at startup beside the
  `roles.yaml` pass, writing a `slack_workspace:<workspace_id>` entity with
  `default_included=false` and an allow rule per listed role — so the app file is the single
  place that says who may drive a workspace.
- The bundled `systemprompt` MCP server stamps its `tools/list` and
  `resources/templates/list` results through core's `build_tool_list_result` and
  `build_resource_template_list_result`, so both carry the SEP-2549 cache metadata
  (`ttlMs`, `cacheScope`) that protocol `2026-07-28` requires. A client that parks
  connectors on a missing stamp now sees the template's server as conformant.
- Helm chart 0.17.0 with appVersion 0.35.0; the CasaOS, DigitalOcean, and Packer
  manifests pin the 0.35.0 image.

### Fixed

- The inline-comment gate globbed only top-level `tests/**/*.rs`, so every nested
  `extensions/**/tests/*.rs` passed without being read -- the exact failure the
  gate exists to prevent. Widened, and the 21 `///` uses it surfaced in test code
  converted to `//`.
- **`examples/pi/setup.sh` aborted depending on the token's length.**
  `_jwt_payload_b64` ended on a bare `[[ $pad -gt 0 ]] && …` test, so it returned 1
  whenever the JWT payload needed no base64 padding; under `set -e` with `pipefail`
  that killed the caller's pipeline right after the session id was extracted, with
  no error message. `trace.sh` carried the same pattern inside a pipeline and
  survived only on payload length. Both are now `if` statements.
- **`setup-local` failed on a clean clone.** `services/slack/example.yaml`
  documented a `link_by_workspace_email` option core does not implement, and the
  config structs are `deny_unknown_fields`, so profile validation refused the whole
  services tree and setup aborted before writing a profile. The field is removed,
  and a new test deserialises every shipped YAML that declares a `ServicesConfig`
  section -- nothing else in the suite reads them, because nothing else runs setup.

## [0.29.0] - 2026-08-05

### Breaking

- **Breaking:** tracks systemprompt-core 0.29.0. `create_router` in the MCP extension takes an `McpSessionRepository`, and the content and gateway-policy jobs take their repositories, in place of a `PgPool`. Migrate by constructing the repository from the pool at the call site and passing it through.
- **Breaking:** `UserService` and `AnalyticsService` are constructed from injected repositories. Migrate by building each repository from the pool and passing them to the constructor.
- **Breaking:** the eval tables are owned by core's `systemprompt-evaluation` extension; the web extension declares them via `cross_extension_tables`. Migrate by deleting any local `SchemaDefinition` for `eval_runs`, `eval_cases`, `eval_results`, `eval_pairs`, `eval_judge_calls`, or `eval_rubrics`; a duplicate owner fails installation with `DuplicateTableOwner`.
- **Breaking:** `ai_requests.context_id` is `NOT NULL`. Rows belonging to no known context carry the sentinel `00000000-0000-0000-0000-4c4547414359`, which analytics reports as no context rather than as a context id.

### Added

- `check-asset-reachability.sh` gate: every shipped front-end asset must be reachable from a template or the asset manifest.
- `check-workspace-deps.sh` gate: every declared workspace dependency must be inherited by at least one member.

### Changed

- Repository functions in `extensions/web/admin` follow the `list_` / `find_` / `get_` return-type convention, enforced by `check-repository-naming.sh`.
- Helm chart 0.10.0 with appVersion 0.29.0; the CasaOS, DigitalOcean, and Packer manifests pin the 0.29.0 image.

### Fixed

- Gateway demo routes whose provider the active profile does not declare are skipped instead of failing.

### Removed

- The unreferenced content-card partial and the emptied `service_plugin_js` asset.

## [0.28.0] - 2026-07-31

### Breaking

- **Breaking:** the governance policy toggle applies on restart rather than reloading in place, because core's `GovernanceEngine::global()` is a `LazyLock`. Migrate by restarting the server after changing `services/governance/config.yaml`.
- **Breaking:** an unparseable `services/governance/config.yaml` fails boot instead of falling back to the built-in defaults. Migrate by validating the file before deploying.

### Changed

- Tracks systemprompt-core 0.28.0. The webhook engine delegates to core's process-wide governance engine, so rate-limit buckets are counted once per request.
- The secrets safety scanner covers egress only; the `secret_scan` governance policy covers the request side.
- Helm chart 0.9.0 with appVersion 0.28.0; the deployment manifests pin the 0.28.0 image.

### Fixed

- The `cli` justfile recipe no longer word-splits quoted arguments.

## [0.27.0] - 2026-07-29

### Breaking

- **Breaking:** the governance policy engine lives in `systemprompt-security`. The four builtin policies, the audit repository and handler, the evaluate handler, and the secrets scanner no longer exist in this repo. Migrate by importing them from `systemprompt_security` and registering third-party policies against core's `GovernancePolicy` trait.
- **Breaking:** `AppPaths::from_profile` takes a `PathResolution`. Migrate by passing the resolution alongside the profile.

### Changed

- `render.yaml` pins `:edge` so boot-time fixes reach the Render service without waiting for a release.
- Helm appVersion 0.27.0; the deployment manifests pin the 0.27.0 image.

### Fixed

- Migration checksum drift is repaired on container boot, so a redeploy no longer needs a manual `just repair-migrations`.

## 0.26.0 — 2026-07-28

### Changed

- Tracks systemprompt-core 0.26.0. The governance webhook supplies the `call_id` that `PolicyContext` now requires: the webhook is this call's only enforcement point and nothing upstream hands it an identity, so it mints one per request. A policy that accumulates state can use it to tell a repeat evaluation of one call from a second call.
- The deployment manifests (Helm, CasaOS, DigitalOcean) pin the 0.26.0 image; the Helm chart is 0.7.0 with appVersion 0.26.0. `render.yaml` tracks `latest` by design and is unchanged.

## 0.14.7 — 2026-06-03

### Fixed

- The `systemprompt` MCP server no longer self-deadlocks on reentrant CLI calls. The tool handler shelled out via the blocking `std::process::Command::output()` from inside its async `handle`, parking a Tokio worker for the lifetime of the child. When the child command was itself one that connects back to the same server (for example `plugins mcp tools --server systemprompt` calling `list_tools`), the parent held a worker waiting on the child while the child waited on the parent's server to answer, so the reentrant call only unblocked when the client's 30s timeout fired and returned an empty tool list. `cli::execute` is now `async` and uses `tokio::process::Command::output().await`, so the parent future yields while the child runs and the reentrant request is serviced normally.

## 0.12.0 — 2026-05-27

### Breaking

- **Aligned with core's split `access_control_entities` + `access_control_rules` schema.** Every direct sqlx call against `access_control_rules` either drops `default_included` from its column list or JOINs `access_control_entities` for it. Template-side `repositories::access_control` (`set_entity_rules`, `bulk_set_rules`) now upserts the catalog row before inserting grants — required by the FK migration 007 added on core. `AccessControlRule` and `AccessControlRuleInput` lose the `default_included` field; dashboard payloads carry it via the entity-level endpoints instead.
- **`gateway_acl::get_default_included` / `set_default_included` replaced** by `gateway_acl::get_entity` (returns `Option<EntityRow>`) and `gateway_acl::upsert_entity(pool, route_id, default_included, source)`. The webhook handler, marketplace filter, gateway catalog, `entity_access`, and `effective` modules transit through the new API; an absent catalog row resolves to `default_included: None`, which the core resolver maps to `DenyReason::UnknownEntity`.
- **Publish pipeline gains an `access_entity_bootstrap` stage** before `acl_yaml_load`. The stage upserts one `access_control_entities` row per `gateway.routes[]` declared in the active profile (`source = "profile:<path>"`). Without this stage the new FK on `access_control_rules` would reject every grant ingested from `services/access-control/`. MCP / agent / skill / plugin / marketplace bootstrapping comes with task #3.
- **Tool-use governance now runs on core's shared `Decision` / `GovernancePolicy` plane.** Every built-in policy (`secret_scan`, `scope_check`, `tool_blocklist`, `rate_limit`) implements `systemprompt_security::policy::GovernancePolicy` and returns the typed `systemprompt_security::authz::Decision` (`Allow { matched_by }` / `Deny { reason: DenyReason::… }`). Audit rows in `governance_decisions.evaluated_rules` are now produced from the typed `DecisionAudit { decision, principal, target, chain }` blob — the previous `serde_json::json!([{rule, result, detail}])` shape is gone. Downstream dashboards or alert rules that decoded the old `evaluated_rules` shape must be updated to the new schema; the top-level columns (`decision`, `policy`, `reason`) are unchanged.
- **`webhook::governance::types::GovernanceContext`, `RuleEvaluation`, `EvaluatedRule`, and `AuditRecord` were removed**, along with `webhook::governance::rules` (replaced by an inlined chain walk inside `webhook::governance::handler`). `Policy` / `PolicyContext` / `PolicyOutcome` no longer exist in `webhook::governance::policy`; the module re-exports `GovernancePolicy` from core. Extensions that registered third-party policies via `inventory::submit!` must switch to the core trait and return `Decision`/`DenyReason` typed values.

## 0.11.2 — 2026-05-25

Aligned with `systemprompt-core` 0.11.2: the gateway model allow-list moves from `services/ai/gateway-policies.yaml` into the profile catalog (`.systemprompt/profiles/<name>/catalog.yaml`).

### Breaking

- **`services/ai/gateway-policies.yaml` no longer carries `allowed_models:`.** Core's `GatewayPolicySpec` has dropped the field; the spec uses `deny_unknown_fields`, so a stale `allowed_models:` will fail boot. Exposed-model declarations move to the profile catalog instead.
- **`endpoint:` and `api_key_secret:` removed from every `gateway.routes[*]` entry.** Both fields now live exclusively on `GatewayProvider` in the catalog; the route references its provider by id and resolves endpoint + secret through the catalog. Core 0.11.2's `deny_unknown_fields` rejects route YAML that still carries them. Operators upgrading from 0.11.1 whose admin UI wrote those fields must strip them before boot — one-shot fix: `yq -i 'del(.gateway.routes[].endpoint) | del(.gateway.routes[].api_key_secret)' .systemprompt/profiles/<name>/config.yaml`. Endpoint + secret are managed at the provider level going forward.
- **`GatewayRouteView` admin DTO drops `endpoint` + `api_key_secret`.** Admin API clients posting `POST /api/admin/gateway/routes` no longer need to send (or can send) these two fields; serde drops them silently on input, and the persisted YAML omits them on output. The companion `validate_route` check loses the inline-secret-prefix detector along with the field it guarded.
- **Two-pass authz on `/v1/messages` (model + route).** The `extensions/web/admin/src/handlers/webhook/governance/authz.rs` webhook now sees both `EntityRef::GatewayModel(ModelId)` and `EntityRef::GatewayRoute(RouteId)` per request — the handler is entity-kind agnostic so no code change, but operators should expect roughly 2× rows in `governance_decisions` per inference call and may want to add model-scoped rules to `access_control_rules` to start exercising the new gate.

### Added

- **Profile gateway catalog (`gateway.catalog_path`)** points at a sibling `catalog.yaml` declaring providers + models (with optional aliases). The dispatcher's `is_model_exposed` gate consults the catalog before route resolution, so a wildcard route (`claude-*`) cannot leak a model the catalog has not declared. Adding a model means editing one file.
- **`just setup-local`** generates the catalog alongside the profile so fresh clones have a consistent baseline.

### Changed

- **`services/ai/gateway-policies.yaml` renamed to `services/gateway/policies.yaml`.** Tracks core's loader path move. The one-release fallback that briefly lived in core was removed before 0.11.2 ships — deployments still on the legacy path MUST move the file before upgrading (see core's 0.11.2 breaking notes).
- **`demo/scenarios/airgap/{02-load.sh,03-governance.sh,architecture.md}`** updated to reflect the new gate ordering, the new policy path, and that policies carry quotas/safety only.
- **`services/content/documentation/gateway-api.md`** points operators at the catalog as the model-exposure surface.
- **`justfile airgap-test` comment** updated to point at the new policy path.

## 0.11.0 — 2026-05-21

Aligned with `systemprompt-core` 0.11. Workspace version bumped from 0.9.2 → 0.11.0.

### Changed

- **Governance policy renamed `secret_injection` → `secret_scan`.** Clean break, no backward compatibility. The policy value emitted into `governance_decisions.policy` is now `secret_scan` (`extensions/web/admin/src/handlers/webhook/governance/policies/secret_scan.rs`). All read paths — repositories (`governance_grp/{portfolio.rs,risk_score.rs}`), the `14_audit_event_notify.sql` trigger, the homepage narratives in `extensions/web/site/src/homepage/demo_scanner/`, and every demo script — were updated to the new name in the same release. The dead `POLICY_SECRET_INJECTION` constant in `extensions/web/admin/src/types/constants.rs` was removed. **Any external dashboard, alert rule, or analytics query pinned to the literal `secret_injection` must be updated to `secret_scan`; historical rows still carrying the old policy string will no longer match any query and will not trigger the `audit_event_notify` breach severity.**

### Added

- **`016_swap_marketplace_admin_owner_to_admin.sql`** seeds the bootstrap `admin` user (`status='active'`, `roles=['admin','user']`) and re-owns the `marketplace-admin` OAuth client to it. Core's `oauth_clients.owner_user_id` NOT NULL constraint (core migration `004_oauth_client_owner`) wiped the synthetic owner introduced in `015_reseed_oauth_client_owner`; this migration replaces it with the real admin row that the scheduler resolves at startup.
- **`017_align_admin_email_with_cli.sql`** aligns the seeded admin's email with what core's CLI local-trial resolver expects (`admin@localhost.dev`). Without this, `admin agents message` would `find_by_email` miss and then collide on the `users.name='admin'` unique key when trying to auto-provision.
- **`015_reseed_oauth_client_owner.sql`** (band-aid kept for upgrade ordering) creates a synthetic `system` user owning `marketplace-admin` so fresh clones get past `010_seed_oauth` once core enforces NOT NULL `owner_user_id`. Superseded by 016 on the next migrate; the row is cleaned up there.

### Fixed

- **`docker/entrypoint.sh`** now runs `systemprompt admin bootstrap` between `infra db migrate` and `infra services start --foreground`. The scheduler refuses to start unless an active admin user resolves; the entrypoint previously assumed a human had run the bootstrap manually.
- **`justfile setup-local`** mirrors the same call after `just migrate`. Fresh clones on a developer machine now get an `admin` user without manual intervention.
- **`demo/00-preflight.sh` Step 0** now pre-checks `.systemprompt/credentials.json` expiry. Expired or absent creds produce a single actionable line and set `CLOUD_OFFLINE=1` for downstream demos; local-profile demos continue normally. Replaces the old behaviour where the cloud-token-expired WARN line was repeated on every CLI invocation throughout the suite.
- **`demo/00-preflight.sh` Step 3** now fails loud when the `/admin/profile` plugin-token scrape returns nothing. The previous silent fallback ("falling back to admin token") wrote the admin-scope JWT to `demo/.token`, so every `scope=service` demo silently degraded to `scope=admin` and analytics filtered on `session_id=plugin_cowork-bundle` returned empty. The fallback was tech debt masking the absence of a plugin-token mint command — see core issue D-4 for the missing `admin keys issue-plugin-token`. Demos that need plugin scope will not run until that command lands.
- `plugins mcp list`, `plugins mcp logs`, `plugins mcp validate`, and `admin agents registry` all work end-to-end against the template clone — the earlier AppPaths-not-initialized, missing-log-file, missing-`--service`, and registry JSON parse errors are gone with core 0.11.
- **`demo/users/01-user-crud.sh`** renamed to `01-user-listing.sh` to match its actual operations (list/count/stats/search only — no C/U/D). Mutating user demos remain isolated per the existing convention (see `04-ip-ban.sh`).

### Security

- Every scheduled job now requires an explicit `owner:` field in `services/scheduler/config.yaml`. The owner is a real admin username — there is no special "system" user. Existing installs must add `owner:` to each job entry; startup fails loudly until they do. The configured owner becomes `JobContext.actor` for every `execute()` call and the principal recorded in audit rows. See `services/content/documentation/authentication.md` for the full attribution model.
- Removed the synthesized `"admin"` fallback in the plugin-env handler. Requests without an authenticated cookie session and without an explicit `user_id` query parameter now return `401 Unauthorized` instead of impersonating the first admin user.
- Replaced the hardcoded `'system'` literal in the secret-migration audit log with the configured job owner. Every row in `secret_audit_log` now traces to a real `users` row.
- Added a `just lint-no-synthesis` guard (wired into `just clippy`) that fails the build if `UserId::new("…")` appears with a string literal in non-allowlisted extension code. Prevents future synthesis from sneaking in.

### Fixed

- `services/plugins/enterprise-demo/config.yaml`: dropped the dead `scripts:` block that referenced two missing files (`demo/01-seed-data.sh`, `demo/sweep.sh`). `core plugins validate` now reports zero errors for this plugin.
- `extension_migrations` tracking-table drift on the `web` extension reconciled (was 15 rows applied vs 11 declared). Migration-status summary now shows clean `11/11`. Clones with the same drift can either run `just repair-migrations` or `DELETE FROM extension_migrations WHERE extension_id = 'web' AND version IN (1, 4, 8, 13)` — the four legacy migrations consolidated out of the source tree.
