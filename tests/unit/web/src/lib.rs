//! Unit tests for `systemprompt-web-shared` pure logic:
//! - `CampaignLink::full_url` UTM query assembly and `?`/`&` separator choice
//! - `BlogConfigValidated::validate` base-URL scheme/parse validation
//! - hook-event ingest leniency, which the governance record depends on
//! - admin display formatting bands and `PageWindow` pagination arithmetic
//! - inventory registry completeness for jobs, renderers, and providers
//! - calendar-month resolution and the month-end P&L's derived figures
//! - Salesforce org drift detection, including the fields no API can read back
//! - the `salesforce` CLI extension: its clap surface (spec default, each
//!   subcommand's flags, repeatable `--user`, rejected input) and the pure
//!   assignee merge behind `apply`
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

#[cfg(test)]
mod builders;
#[cfg(test)]
mod campaign_link_full_url;
#[cfg(test)]
mod config_base_url;
#[cfg(test)]
mod config_errors;
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
mod format_display;
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
mod page_window;
#[cfg(test)]
mod paper_metadata;

mod pii_scanner;
#[cfg(test)]
mod registry_completeness;
#[cfg(test)]
mod report_pnl;
#[cfg(test)]
mod salesforce_cli_logic;
#[cfg(test)]
mod salesforce_cli_parse;
#[cfg(test)]
mod salesforce_org_diff;
#[cfg(test)]
mod salesforce_org_package;
#[cfg(test)]
mod salesforce_org_secrets;
#[cfg(test)]
mod secrets_scanner_response;

#[cfg(test)]
mod seed_contract;
#[cfg(test)]
mod shared_errors;
#[cfg(test)]
mod shared_ids;
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
mod site_skills_grouping;
#[cfg(test)]
mod site_skills_prerenderer;
mod spend_forecast;
#[cfg(test)]
mod web_extension_providers;
#[cfg(test)]
mod web_extension_wiring;
#[cfg(test)]
mod web_schemas;
