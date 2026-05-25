# Changelog

## Unreleased

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
