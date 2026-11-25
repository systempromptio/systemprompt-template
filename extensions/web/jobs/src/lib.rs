//! The web extension's publish pipeline.
//!
//! Every job in this crate implements the core `Job` trait and is contributed
//! by type in `WebExtension::contribute` through [`contribute_jobs`]:
//!
//! - **Publish pipeline** ([`PublishPipelineJob`]) — runs at server startup and
//!   orchestrates asset copy, content ingestion, prerender, and robots/llms.txt
//!   generation. Sub-jobs are individually addressable via the CLI for targeted
//!   re-runs.
//! - **Build helpers** ([`BundleAdminCssJob`], [`CopyExtensionAssetsJob`]) —
//!   emit the static surface under `web/dist/` consumed by the SSR layer.
//!   Content prerendering is core's `content_prerender` job; the pipeline runs
//!   the same prerender out of turn. The asset list is handed in by the
//!   composing extension, so this crate never names the admin plane.
//!
//! The governance, access, usage and analytics jobs belong to the admin plane
//! and are contributed by `systemprompt_web_admin::jobs`.
//!
//! Errors normalise on [`JobError`]; the scheduler logs and surfaces them
//! through `infra logs trace`.

mod error;
mod registry;

mod bundle_admin_css;
mod copy_assets;
mod ingestion;
mod llms_txt;
mod publish;
mod robots;

pub use error::JobError;
pub use registry::contribute_jobs;

pub use bundle_admin_css::BundleAdminCssJob;
pub use copy_assets::{AssetDeclaration, CopyExtensionAssetsJob};
pub use ingestion::ContentIngestionJob;
pub use llms_txt::LlmsTxtGenerationJob;
pub use publish::PublishPipelineJob;
pub use robots::RobotsTxtGenerationJob;

/// The pure helpers behind the jobs above, re-exported for the external test
/// workspace so their file-format behaviour can be asserted without a
/// scheduler, a database, or an `AppPaths`. Not part of the public API — the
/// job structs are.
#[doc(hidden)]
pub mod internals {
    pub use crate::bundle_admin_css::{collect_css_files, concatenate_css_files};
    pub use crate::copy_assets::{copy_all_assets, copy_asset};
    pub use crate::llms_txt::{sort_entries_in_place, write_header, write_section};
    pub use crate::publish::PipelineStats;
    pub use crate::robots::build_robots_txt_content;
}
