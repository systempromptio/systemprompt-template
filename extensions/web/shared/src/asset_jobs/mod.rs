//! The console asset jobs every flavour contributes.
//!
//! [`BundleAdminCssJob`] concatenates the console stylesheets and
//! [`CopyExtensionAssetsJob`] copies the web member's declared assets into
//! `web/dist/`. They live here, below both the console and the publish
//! pipeline, so a flavour without the public site still serves a styled
//! console and the pipeline runs the same steps out of turn.

mod bundle_admin_css;
mod copy_assets;

use thiserror::Error;

pub use bundle_admin_css::{BundleAdminCssJob, collect_css_files, concatenate_css_files};
pub use copy_assets::{AssetDeclaration, CopyExtensionAssetsJob, copy_all_assets, copy_asset};

#[derive(Debug, Error)]
pub enum AssetJobError {
    #[error("asset file operation failed")]
    Io(#[from] std::io::Error),
    #[error("job context missing a required value")]
    MissingContext(#[from] systemprompt::traits::MissingDependency),
    #[error("failed to read {failed} CSS file(s) during bundling")]
    CssUnreadable { failed: u64 },
}

impl From<AssetJobError> for systemprompt::traits::ProviderError {
    fn from(err: AssetJobError) -> Self {
        Self::Internal(Box::new(err))
    }
}
