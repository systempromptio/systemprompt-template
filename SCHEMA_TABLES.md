# Database tables — Enterprise Demo (post-cleanup)

Snapshot of the live demo DB (`local` profile, 2026-06-30) after the schema bloat
cleanup that removed 10 dead tables. See `extensions/web/schema/migrations/018_drop_dead_tables.sql`
and `../systemprompt-core/table.md` for the core follow-ups.

**Totals:** 89 base tables (was 99) · 41 views · 5 `marketplace.*` tables.

The earlier "139 tables" figure counted `public` base tables **and** views together
(`information_schema.tables`). It was never 139 base tables.

---

## Tables with data (32)

| Table | Rows | Size |
|---|--:|---|
| governance_decisions | 2,167 | 3664 kB |
| logs | 800 | 1304 kB |
| engagement_events | 217 | 232 kB |
| user_sessions | 102 | 784 kB |
| extension_migrations | 67 | 80 kB |
| plugin_usage_events | 44 | 176 kB |
| user_activity | 24 | 96 kB |
| access_control_rules | 21 | 80 kB |
| analytics_events | 20 | 232 kB |
| mcp_sessions | 19 | 144 kB |
| markdown_content | 15 | 392 kB |
| markdown_content_enrichment | 15 | 280 kB |
| files | 14 | 168 kB |
| plugin_session_summaries | 14 | 112 kB |
| access_control_entities | 13 | 48 kB |
| mcp_artifacts | 12 | 296 kB |
| mcp_tool_executions | 12 | 400 kB |
| task_execution_steps | 12 | 64 kB |
| plugin_usage_daily | 6 | 96 kB |
| users | 6 | 112 kB |
| anomaly_thresholds | 5 | 32 kB |
| scheduled_jobs | 5 | 96 kB |
| ai_request_messages | 3 | 152 kB |
| oauth_client_redirect_uris | 3 | 48 kB |
| services | 3 | 64 kB |
| oauth_client_grant_types | 2 | 64 kB |
| oauth_client_scopes | 2 | 64 kB |
| ai_gateway_policies | 1 | 64 kB |
| ai_requests | 1 | 336 kB |
| departments | 1 | 48 kB |
| oauth_client_response_types | 1 | 48 kB |
| oauth_clients | 1 | 64 kB |

---

## Empty tables (57)

These are empty in the demo but **wired into real code** — they populate only when the
relevant flow is exercised (passkey login, OAuth refresh, A2A artifacts, quota, safety
scans, bridge prefs, link tracking, etc.). They are **not** dead code; the demo just
never triggered them.

**Auth / OAuth / WebAuthn / Bridge**
`oauth_auth_codes`, `oauth_refresh_tokens`, `oauth_state_bindings`, `oauth_jti_revocations`,
`oauth_client_contacts`, `id_jag_replay`, `webauthn_credentials`, `webauthn_challenges`,
`webauthn_setup_tokens`, `user_device_certs`, `federated_identities`, `user_api_keys`,
`bridge_sessions`, `bridge_exchange_codes`, `bridge_user_host_prefs`,
`bridge_user_host_model_prefs`, `banned_ips`

**AI gateway / requests**
`ai_requests` *(1 row)*, `ai_request_payloads`, `ai_request_tool_calls`,
`ai_safety_findings`, `ai_quota_buckets`

**Agents / A2A tasks**
`agent_tasks`, `task_messages`, `task_artifacts`, `task_push_notification_configs`,
`message_parts`, `artifact_parts`, `context_agents`, `context_notifications`,
`user_contexts`

**Content / marketing / analytics**
`campaign_links`, `link_clicks`, `content_performance_metrics` ⚠, `content_files`,
`markdown_categories`, `markdown_fts`, `funnels` ⚠, `funnel_steps` ⚠, `funnel_progress` ⚠,
`fingerprint_reputation`

**Admin dashboard / sessions / profiles**
`admin_traffic_reports`, `daily_summaries`, `session_analyses`, `session_entity_links`,
`session_ratings`, `skill_ratings`, `session_transcripts` ⚠, `tenant_activity` ⚠,
`user_settings`, `user_profile_reports`, `user_profile_ext`, `device_app_links`

**Secrets**
`plugin_env_vars`, `secret_audit_log`, `secret_resolution_tokens`, `user_encryption_keys`

**Infra**
`event_outbox`

> ⚠ = **incomplete feature, kept** — has read code but no writer yet. Flagged for a
> follow-up producer rather than deletion. Core ones (`funnels*`, `content_performance_metrics`,
> `webauthn_challenges`) are documented in `../systemprompt-core/table.md`; the
> template ones (`tenant_activity` = cloud-sync surface, `session_transcripts` =
> transcript viewer) await ingest writers.

---

## Removed in this cleanup (10 — were dead)

`employee_xp_ledger`, `employee_ranks`, `employee_achievements`, `employee_daily_usage`
(gamification, seed-only), `hook_catalog`, `hook_plugins`, `hook_files` (schema-only),
`skill_secrets` (cascade stub only), `user_ranks`, `user_achievements` (dead gamification
re-platform).

---

## Views (41)

`v_active_anonymous_sessions`, `v_ai_crawler_activity`, `v_ai_image_generation_stats`,
`v_ai_scraper_activity`, `v_all_activity`, `v_behavioral_bot_analysis`,
`v_bot_human_metrics_comparison`, `v_bot_traffic_summary`, `v_bot_type_breakdown`,
`v_campaign_daily_performance`, `v_campaign_performance`, `v_clean_human_traffic`,
`v_clean_traffic`, `v_client_conversion_rates`, `v_client_errors`, `v_client_rate_limits`,
`v_content_journey`, `v_conversion_funnel`, `v_daily_conversions`, `v_engaged_traffic`,
`v_high_risk_fingerprints`, `v_landing_page_conversion`, `v_link_click_stream`,
`v_link_performance`, `v_link_performance_by_country`, `v_link_performance_by_device`,
`v_log_analytics_by_client`, `v_preconversion_engagement`, `v_recent_bot_activity`,
`v_referrer_landing_flow`, `v_scanner_activity`, `v_security_threats`,
`v_seo_crawler_activity`, `v_session_analytics_by_client`, `v_source_content_performance`,
`v_time_to_conversion`, `v_top_performing_links`, `v_top_referrer_sources`,
`v_traffic_composition`, `v_traffic_source_quality`, `v_utm_campaign_performance`

Bot/traffic views read the populated `logs` / `user_sessions` and return data. The
conversion/funnel views (`v_conversion_funnel`, `v_landing_page_conversion`,
`v_time_to_conversion`, `v_daily_conversions`, `v_preconversion_engagement`) read the
unwritten `funnels*` / `content_performance_metrics` tables and return zero rows — they
go live only if those features get writers.

---

## `marketplace` schema (5 — Paddle billing, separate schema)

`marketplace.plans`, `marketplace.paddle_customers`, `marketplace.subscriptions`,
`marketplace.paddle_webhook_events`, `marketplace.magic_link_tokens`
