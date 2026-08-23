# Changelog

All notable changes to this repository are recorded here, newest first.

Conventions (strict — hold every entry to them):

- Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/): an `## Unreleased`
  section at the top, then one `## <version> — <YYYY-MM-DD>` section per release, each with
  only the categories it needs, in this order: `### Breaking`, `### Added`, `### Changed`,
  `### Fixed`, `### Removed`.
- Entries are written for the reader who did not make the change: full sentences, what changed
  and **why**, named files/commands/flags where the reader will need them. No bare "updated X".
- Every user-visible or operator-visible change lands in `Unreleased` **in the same commit** as
  the change itself; internal-only refactors are recorded when they alter an API another crate,
  config, or dashboard consumes.
- A release moves the `Unreleased` content under its version heading; `Unreleased` is never
  deleted, only emptied.
- Version numbers track the root workspace `version` in `Cargo.toml`; bridge-only releases are
  prefixed `bridge-` (e.g. `bridge-0.17.1`).

## Unreleased

### Changed

- Tracks systemprompt-core 0.36.0. Pin-only: the breaking
  `McpDomainError::PortHolderUnverifiable` variant is not matched in this repo, and
  the messaging and Slack APIs 0.36.0 changed are not used here.

### Added

- The bundled `systemprompt` MCP server stamps `tools/list` and
  `resources/templates/list` through core's `build_tool_list_result` and
  `build_resource_template_list_result`, so both carry the SEP-2549 cache metadata
  (`ttlMs`, `cacheScope`) that protocol `2026-07-28` requires. A client that parks
  connectors on a missing stamp now sees this server as conformant.

### Fixed

- The inline-comment gate walked only top-level `tests/**/*.rs`, so every nested
  `extensions/**/tests/*.rs` passed vacuously. Widening the glob surfaced 22 real
  `///` uses in test code, now converted, and two clippy failures that had been
  invisible to the gate.

### Breaking

- **The web extension's migration chain is retired: a clean database bootstraps from the
  declarative schema and boot seeds alone.** All 22 `extensions/web/schema/migrations/*.sql`
  files are deleted. Core ≥0.32 stamps migrations as applied on fresh installs instead of
  executing them, so migration files only ever ran on established databases — and every known
  deployment (including the production Fly tenant, verified at head) had applied them; orphaned
  `extension_migrations` rows are inert. `departments` moves from `12_management.sql` to
  `16_organizations.sql` with its final shape (`org_id NOT NULL`, `departments_org_fk`,
  `idx_departments_org_name` — the names the old backfill migration created, so fresh and
  established databases converge). Boot seeds grow to three insert-if-absent files
  (`admin_oauth_client`, `marketplace_plans`, `house_organization`); the seed-contract test
  enforces both the manifest and `ON CONFLICT … DO NOTHING`. Demo tenants, users, and traffic
  leave the boot path entirely: they are seeded only by the demo flow
  (`demo/02-seed-demo-tenants.sh` + `demo/fixtures/demo-tenants.sql`), and
  `services/access-control/plans.yaml` no longer recreates the demo organizations on every
  boot. This removes the seed-data-in-migrations pattern whose checksum-drift/repair-replay
  failure mode caused the 2026-08-19 production boot loop.

### Added

- **Slack is now an inbound surface, restricted to admins.** `/systemprompt <cli command>` in
  Slack runs against this instance and answers in the channel. `services/slack/astound.yaml`
  declares the app (signing secret and bot token by reference, the bot shared with the existing
  outbound alerting in `slack_alerts.rs`) and routes the slash-command key — not a channel id,
  because core dispatches on every `message` in a routed channel, not only `app_mention`. Four
  gates stand between a Slack message and a CLI call, each denying by default: the workspace
  rule projected from `authz.allowed_roles`, the sender's identity, the scope of the token core
  mints for them, and the MCP server's own audience/scope/role requirements. A non-admin gets an
  ephemeral `⛔`, audited like any other refusal.
- **`services/agents/admin_console.yaml` — the first A2A agent this instance ships.** The
  inbound messaging pipeline can only dispatch to an agent; this one carries the `systemprompt`
  MCP server and nothing else, runs on port 9101, and declares `oauth.scopes: [admin]` so a
  token minted for a non-admin sender is refused at the A2A door. It is bound into the
  `astound-admin` plugin and declares its own admin-only rule in `roles.yaml`, which opts it out
  of the marketplace's `[user]` cascade.
- **`POST|DELETE /api/public/admin/users/{user_id}/slack-identity`** — link or detach the Slack
  account a user drives the platform from, for accounts whose Slack profile carries an
  unconfirmed or different email and so cannot take the automatic path. Body:
  `{"slack_user_id": "U…"}`. Writes the same `federated_identities` row the Salesforce Connect
  flow uses, under `https://slack.com`, and refuses to steal a mapping owned by another user.

### Changed

- **A Slack app's `authz.allowed_roles` is now enforced, not just documented.** Core wrote the
  projection (`ingest_slack_apps`) but nothing ever called it, so the field described an
  intention no rule backed. `repositories/config/acl_yaml_loader.rs` now runs it at startup
  beside the `roles.yaml` pass, writing a `slack_workspace:<workspace_id>` entity with
  `default_included=false` and an allow rule per listed role. The Slack app file therefore stays
  the single place that says who may drive a workspace — `roles.yaml` deliberately carries no
  `slack_workspace` rules, only a pointer.

- **The `astound-dev` plugin (v2.0.0) now ships the full Astound development suite** — 68 skills,
  mirroring the team's [sfcc-next-cursor](https://github.com/Astound-Digital/sfcc-next-cursor)
  Cursor tooling repo. 62 skills are ported 1:1 into `services/skills/` (snake_case ids:
  `b2c-hooks` → `b2c_hooks`), spanning B2C build (`b2c_*`, incl. the 435-file offline `dw.*`
  corpus in `sfcc_api_classes`), Storefront Next (`sfnext_*`), operations (`atlassian`,
  `b2c_config`, `b2c_logs`, …), release (`git_commit`, `b2c_code`, `b2c_mrt`, …), and test
  (`playwright_cli`, `code_review`, `a11y_audit`, `systematic_debugging`). Because this instance
  ships skills rather than agents, the Cursor repo's 22 non-OpenSpec rules are folded into the
  new `dev_rules` skill and its 12 relevant agents/commands are folded into the skills they
  orchestrated (code-review + security-auditor → `code_review`, git-manager → `git_commit`,
  diagnosis/fix/fix-verification → `systematic_debugging`, perf-optimizer →
  `sfnext_performance`, scapi-cartridge-dev → `b2c_scapi_custom`, planning/feature-conductor/
  implement → the `dev_plan`/`dev_build` entry skills). The OpenSpec/opsx, project-context, and
  eval machinery was deliberately not ported. The four existing `dev_*` skills remain as
  entry-point routers with their `## Astound rules` drop-in sections now filled.
- **Four admin user-management skills in the `astound-admin` plugin**, all driving the
  admin-only `systemprompt` MCP CLI passthrough: `manage_users` (create/update/delete/merge/
  bulk/export), `block_users` (account suspension + IP bans — note `admin users ban` is keyed to
  an IP address, not a user), `manage_roles` (promote/demote/assign, with the
  sign-out-and-back-in token-reissue caveat), and `manage_sessions` (list/force-end/cleanup).
  Destructive commands document the confirm-then-`--yes` guardrail the CLI enforces. Each skill
  carries an admin-only entity rule in `services/access-control/roles.yaml`.

### Changed

- **Adopted systemprompt-core 0.32.1 (published).** Patch bump via
  `scripts/sync-release-version.sh 0.32.1` (workspace version, `tests/` pins, image tags);
  lockfiles re-resolve the published crates. Carries core's `admin bootstrap` owner-email fix
  (explicit `system_admin.email`; already-bootstrapped installs unaffected) and the bridge
  browser-launch stdio fix, so both bridge binaries were rebuilt and redeployed.
- **Adopted systemprompt-core 0.32.0 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.31.0 to 0.32.0, both `[patch.crates-io]` blocks are
  re-commented (marker: `# INACTIVE: core 0.32.0 is published`) so the build resolves the
  published crates, lockfiles re-resolve against crates.io, and the `.sqlx` offline caches were
  regenerated (936 queries). Adoption fallout across the extensions tracks core's 0.32 API:
  the CLI session store's `load`/`load_or_reset` split, the Salesforce CLI's staged bootstrap
  errors (`stage_err`), and assorted admin/jobs error-adaptation sites now annotated under the
  tightened `check-http-errors` gate.

- **Adopted systemprompt-core 0.31.0 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.30.1 to 0.31.0 via
  `scripts/sync-release-version.sh 0.31.0` (workspace version, `tests/` pins, Helm chart 0.9.0
  with appVersion 0.31.0, and the CasaOS/DigitalOcean image tags), with both `[patch.crates-io]`
  blocks left commented so the build resolves the published crates. No adoption fallout: the
  extensions already compile against the 0.31 API surface and the `.sqlx` offline caches
  validate unchanged.
- **Adopted systemprompt-core 0.30.1 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.29.0 to 0.30.1 via
  `scripts/sync-release-version.sh 0.30.1` (workspace version, `tests/` pins, Helm chart 0.8.0
  with appVersion 0.30.1, and the CasaOS/DigitalOcean image tags), with both `[patch.crates-io]`
  blocks left commented so the build resolves the published crates, not the 0.31-in-progress
  sibling. Adoption fallout handled per `docs/RELEASING.md` Step A0: the `settings:` block in
  `services/config/config.yaml` moves to core's snake_case keys (`agent_port_range`,
  `mcp_port_range`, `auto_start_enabled`, `schema_validation_mode` — the camelCase forms now
  fail boot validation), migrations ran with the new binary, and the `.sqlx` offline caches were
  regenerated (932 queries).
- Three shared files re-synced with the template sibling to satisfy the fork-drift gate: the
  `systemprompt` MCP server's text fallback pairs the artifact with a `Ran `<command>``
  summary so structured clients don't see stdout twice, SSR render failures convert through the
  new `From<AdminTemplateError> for AdminError` instead of a formatted internal error, and a
  duplicated comment token in the governance authn handler is gone. Integration tests updated to
  core 0.30.1's wire shape, where a tool result's text block carries `summary\n\nbody`.
- `admin_user_report` is now strictly read-only: its role-mutation commands moved to
  `manage_roles`/`manage_users`, and its `session list` invocation was corrected to the CLI's
  positional form (`admin users session list <user-id>`).
- The `enterprise-demo` marketplace catalogue includes all 67 new skills; the marketplace
  version is unchanged, so the checked-in marketplace JSON needed no regeneration (the
  validation gate pins plugin list + marketplace version only).
- Both MCP server extensions (`systemprompt`, `knowledge-bank`) track core's current MCP API:
  output schemas come from `McpOutputSchema::validated_schema()` instead of
  `ToolResponse::schema()`, and tool dispatch threads the new `ClientProfile` through a
  `Dispatch` context struct. Integration tests updated to match.
- Contact email migrated from `ed@tyingshoelaces.com` to `ed@systemprompt.io` everywhere it was
  published: workspace `authors` (root + bridge), plugin author blocks, the web theme
  `support_email`, `AGENTS.md`, docs, examples, and the recorded demo SVGs.
- `scripts/build-coordinator.sh` fingerprints the `bridge/` tree, so bridge edits correctly
  invalidate coordinated build results.

### Removed

- **The repo is slimmed to what this instance actually deploys.** The Helm chart (`helm/`),
  the DigitalOcean/CasaOS deploy scenarios (`deploy/` now keeps only `clean-client/` and
  `salesforce/`), the `demo/` recording, scenario, architecture, and governance suites
  (`demo/` now keeps the preflight, seed, and fixtures used by live flows), and the
  `docs/install/` tree are deleted. The preflight source-gate suite grows to 23 gates and
  `preflight-full` adds a cargo-hack pass; `scripts/check-file-size.sh` and friends tolerate
  the removed directories.

- ~~`coverage/baseline.json`: the tracked coverage ratchet baseline is retired.~~ Reverted in
  the same release: `coverage-check` (part of `just preflight`) hard-fails without a baseline,
  so retiring the file silently disabled the ratchet. The baseline is re-recorded at the current
  coverage instead — total 82.51% (up from 81.17), global floor restored at 80.0.
- Stale `Unreleased` bullets describing the systemprompt-core 0.22→0.23 pin bump — that upgrade
  shipped long ago and is superseded by the released `0.26.0` entry below.

## bridge-0.17.1 — 2026-07-23

### Fixed

- The Windows bridge GUI now actually renders styled. The webview served itself from `http://sp.app`; the `.app` TLD is HSTS-preloaded in Chromium, so WebView2 force-upgraded every stylesheet/script request to `https://sp.app`, bypassing wry's `http://sp.*` interception filter — the document rendered but every subresource died on the real network. Core now serves the Windows GUI over `https://sp.app` with wry's `with_https_scheme`, so the HSTS upgrade is a no-op and assets are intercepted normally.

## bridge-0.17.0 — 2026-07-23

### Fixed

- The Windows bridge splash no longer renders as raw unstyled HTML. Core's generated GUI asset manifest keyed entries with host filesystem paths, so a native Windows build produced backslash keys (`css\main.css`) that never matched the webview's forward-slash URL lookups — every stylesheet and script 404'd silently. Core now normalizes manifest keys to URL form, fails the build on a backslash key or a missing `css/main.css`, and logs asset 404s instead of serving them silently.
- The Windows bridge no longer opens to a blank window. Core's GUI asset router missed the new session service module; core now generates its routing table from the staged web tree, and this repo's overlay opts out of core's Windows resource embed (`SYSTEMPROMPT_BRIDGE_WINRES=off`) so the branded icon is the only one linked.
- The bridge setup overlay is rebased onto core's current GUI: it gains the one-way setup latch and settled-snapshot guard (removes the splash flicker during startup probing), the preserved logo slot required by core's DOM reconciler, locale fallbacks in the gateway probe strings, and the splash-to-app fade.

## 0.14.8 — 2026-07-03

### Removed

- The vestigial pre-generated `storage/files/plugins/` tree and the `just marketplace` recipe (`core plugins generate`) that produced it. The bridge plugin-file endpoint assembles every bundle live from the `build_plugin_bundle` pipeline — the same bytes the gateway hashes into the signed manifest — so the static tree was never served and only invited drift from the manifest hash.
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
