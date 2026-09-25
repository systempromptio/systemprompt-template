//! Unit tests for `systemprompt-web-shared` pure logic:
//! - `CampaignLink::full_url` UTM query assembly and `?`/`&` separator choice
//! - `BlogConfigValidated::validate` base-URL scheme/parse validation
//! - hook-event ingest leniency, which the governance record depends on
//! - admin display formatting bands and `PageWindow` pagination arithmetic
//! - inventory registry completeness for jobs, renderers, and providers
//! - calendar-month resolution and the month-end P&L's derived figures
//! - the `secrets` gateway scanner's response surface, which must cover tool
//!   calls and unmodelled blocks, not only `Text`
//! - the shared value layer the other web crates agree on: id newtypes, the
//!   error enums' code/status/retryability, accumulating config errors, the
//!   content/link wire types, the creation-parameter builders, and the
//!   validated content-source view
//! - the public site (`systemprompt-web-site`): date formatting, the docs
//!   learning block and children-card escaping, the curated skills-page
//!   category order, the asset manifest, the YAML config defaults, the shared
//!   partial registrations, and the non-fatal config loader
//! - the `web` extension facade itself: its metadata, dependencies, asset
//!   manifest, inventory-backed provider getters, and schema/migration lists
//! - the content crate's editorial gate (which metadata faults block a publish
//!   and which only warn), its update-parameter builder, short-code shape, and
//!   API envelopes
//! - the jobs crate's pure halves: robots.txt / llms.txt byte format, CSS
//!   bundle ordering, asset copy's required-vs-optional split, the boot-time
//!   governance config refusal, and the job error/tally plumbing
//! - the admin crate's pure halves, migrated here from its former in-crate
//!   `tests/` dir: ADFS assertion claims and the group->role map, the gateway
//!   catalog/budget surfaces, marketplace catalog and hook shapes, the
//!   governance webhook, secrets crypto, the Handlebars engine and its helpers,
//!   activity/trace constructors, analytics redaction, and the id/range/SVG
//!   value layer
//! - the console landing page's two pure rules: when an MCP server counts as
//!   alive from its session heartbeat, and the polarity of a window-over-window
//!   delta
//! - the developer login link's gate, code hash, and printed URL
//! - the `project_manager` semi-admin role: which role sets reach the admin
//!   dashboard, the AD group glob that grants it, and the project/history
//!   scopes that widen for it
//! - the two front-end gates that used to be shell scripts and then in-crate
//!   tests: admin template/CSS agreement and the textual front-end standards,
//!   plus the asset-manifest check, all sharing `support`

#[cfg(test)]
mod campaign_link_full_url;
#[cfg(test)]
mod catalog_sorting;
#[cfg(test)]
mod config_base_url;
#[cfg(test)]
mod config_errors;
#[cfg(test)]
mod console_role;
#[cfg(test)]
mod content_api_types;
#[cfg(test)]
mod content_config_paths;
#[cfg(test)]
mod content_models;
#[cfg(test)]
mod content_short_code;
#[cfg(test)]
mod content_sources;
#[cfg(test)]
mod content_update_params;
#[cfg(test)]
mod content_validation;
#[cfg(test)]
mod content_validation_results;
#[cfg(test)]
mod conversation_view;
#[cfg(test)]
mod dev_login_pure;
#[cfg(test)]
mod devices_page;
#[cfg(test)]
mod doc_links;
#[cfg(test)]
mod downstream_credential_configs;
#[cfg(test)]
mod format_display;
#[cfg(test)]
mod groups_yaml_types;
#[cfg(test)]
mod release_version_substitution;
#[cfg(test)]
mod role;

#[cfg(test)]
mod conversation_sort;
#[cfg(test)]
mod gateway_policy_config;
#[cfg(test)]
mod gateway_text_markers;
#[cfg(test)]
mod history_scope;
#[cfg(test)]
mod hook_event_dispatch;
#[cfg(test)]
mod html_escape;
#[cfg(test)]
mod jobs_assets_copy;
#[cfg(test)]
mod jobs_bundles;
#[cfg(test)]
mod jobs_errors_stats;
#[cfg(test)]
mod scope;
#[cfg(test)]
mod scope_attribution;

#[cfg(test)]
mod jobs_finops;
#[cfg(test)]
mod jobs_governance_config;
#[cfg(test)]
mod jobs_metadata;
#[cfg(test)]
mod jobs_robots_llms;
#[cfg(test)]
mod link_models;
#[cfg(test)]
mod month_range;
#[cfg(test)]
mod overview;
#[cfg(test)]
mod page_window;
#[cfg(test)]
mod paper_metadata;

#[cfg(test)]
mod pii_scanner;
#[cfg(test)]
mod registry_completeness;
#[cfg(test)]
mod report_pnl;

#[cfg(test)]
mod seed_contract;
#[cfg(test)]
mod shared_errors;
#[cfg(test)]
mod short_id_display;
#[cfg(test)]
mod site_assets;
#[cfg(test)]
mod site_config_loader;
#[cfg(test)]
mod site_configs;
#[cfg(test)]
mod site_docs_learning;
#[cfg(test)]
mod site_docs_page_data;
#[cfg(test)]
mod site_docs_provider;
#[cfg(test)]
mod site_format_date;
#[cfg(test)]
mod site_org_url_extender;
#[cfg(test)]
mod site_page_providers;
#[cfg(test)]
mod site_partials;
#[cfg(test)]
mod site_release_version_extender;
#[cfg(test)]
mod site_skills_grouping;
#[cfg(test)]
mod site_skills_prerenderer;
#[cfg(test)]
#[cfg(test)]
mod web_extension_providers;
#[cfg(test)]
mod web_extension_wiring;
#[cfg(test)]
mod web_schemas;

// Migrated from the former in-crate `extensions/web/admin/tests/` and
// `extensions/web/tests/` directories: the tests workspace is the only home.
#[cfg(test)]
mod access_control_yaml_source;
#[cfg(test)]
mod account_pages;
#[cfg(test)]
mod activity_constructors;
#[cfg(test)]
mod adfs_claims;
#[cfg(test)]
mod adfs_config_files;
#[cfg(test)]
mod adfs_session_pure;
#[cfg(test)]
mod adfs_state_cookie;
#[cfg(test)]
mod admin_css_classes;
#[cfg(test)]
mod ai_catalog_config;
#[cfg(test)]
mod analytics;
#[cfg(test)]
mod analytics_conversations_redact;
#[cfg(test)]
mod asset_manifest;
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
mod config_gateway_pure;
#[cfg(test)]
mod frontend_standards;
#[cfg(test)]
mod gateway_catalog_pure;
#[cfg(test)]
mod governance_decision_view;
#[cfg(test)]
mod governance_warnings_rollup;
#[cfg(test)]
mod hooks_track_ai_pure;
#[cfg(test)]
mod hooks_track_commits_pure;
#[cfg(test)]
mod hooks_track_loc_pure;
#[cfg(test)]
mod marketplace_catalog_pure;
#[cfg(test)]
mod marketplace_hooks_pure;
#[cfg(test)]
mod plugins_env_unauth;
#[cfg(test)]
mod profile_schema;
#[cfg(test)]
mod projects_page;
#[cfg(test)]
mod secrets_crypto_pure;
#[cfg(test)]
mod statusline_ingest_pure;
#[cfg(test)]
mod support;
#[cfg(test)]
mod template_engine;
#[cfg(test)]
mod template_helpers;
#[cfg(test)]
mod template_parse;
#[cfg(test)]
mod traces_analytics_pure;
#[cfg(test)]
mod types_roundtrip;
#[cfg(test)]
mod users_page;
#[cfg(test)]
mod util_ranges;
#[cfg(test)]
mod util_svg_pure;

#[cfg(test)]
mod india_skills;
#[cfg(test)]
mod salesforce_orgs;

mod bridge_release_parity;

#[cfg(test)]
mod vertex_rate_card_routes;


#[cfg(test)]
mod ingestion_identity;

#[cfg(test)]
mod managed_assets;

#[cfg(test)]
mod managed_bundle;
