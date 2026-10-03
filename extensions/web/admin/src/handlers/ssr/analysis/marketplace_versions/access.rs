//! Who may read which marketplace's version history.

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceVersionMetricsRow, VersionWindow, list_marketplace_version_metrics,
};
use crate::types::UserContext;

// Why: version history is a console reading. This instance has no
// marketplace-participant tier, so a caller without console access has no
// version history to look at.
pub(crate) fn require_versions_reader(user: &UserContext) -> AdminResult<()> {
    if !user.is_console {
        return Err(AdminError::Forbidden(
            "Console access required for version history".to_owned(),
        ));
    }
    Ok(())
}

// Why: kept as the one question every version reader asks, so the export
// datasets and the pages cannot disagree. Without a participant tier the
// answer is the console flag; the marketplace id is part of the question so a
// finer rule lands here and nowhere else.
pub(crate) const fn may_read_marketplace(
    user: &UserContext,
    _marketplace_id: &MarketplaceId,
) -> bool {
    user.is_console
}

// Why: a marketplace the caller may not read gets the same not-found as one
// that was never recorded, so the URL says nothing about which exist.
pub(super) async fn readable_versions(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
    user: &UserContext,
) -> AdminHtmlResult<Vec<MarketplaceVersionMetricsRow>> {
    let rows = if may_read_marketplace(user, marketplace_id) {
        list_marketplace_version_metrics(pool, window, marketplace_id).await?
    } else {
        Vec::new()
    };
    if rows.is_empty() {
        return Err(AdminError::NotFound(format!(
            "No version of marketplace '{marketplace_id}' has been recorded"
        ))
        .into());
    }
    Ok(rows)
}
