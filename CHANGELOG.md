# Changelog

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
