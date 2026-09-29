//! The data behind whichever Versions tab is open. Only the active tab's
//! reads run; every other view stays `None`.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use super::{Tab, VersionsQuery, clean_hash, compare, distribution, evaluation, history};
use crate::error::AdminResult;
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceVersionMetricsRow, VersionCompletionRow, VersionWindow,
};
use crate::repositories::analysis::plugin_eval::{list_plugin_eval_runs, list_plugin_eval_tools};
use crate::routes::managed_state::ManagedState;

pub(super) struct TabInput<'a> {
    pub tab: Tab,
    pub window: VersionWindow,
    pub marketplace_id: &'a MarketplaceId,
    pub rows: &'a [MarketplaceVersionMetricsRow],
    pub scores: &'a [VersionCompletionRow],
    pub history: &'a [history::VersionView],
}

#[derive(Default)]
pub(super) struct TabViews {
    pub evaluation: Option<evaluation::EvaluationView>,
    pub compare: Option<compare::CompareView>,
    pub distribution: Option<distribution::DistributionPage>,
}

pub(super) async fn tab_views(
    pool: &PgPool,
    managed: &Arc<ManagedState>,
    input: TabInput<'_>,
    query: VersionsQuery,
) -> AdminResult<TabViews> {
    let wanted = (clean_hash(query.a)?, clean_hash(query.b)?);
    Ok(match input.tab {
        Tab::History => TabViews::default(),
        Tab::Evaluation => TabViews {
            evaluation: Some(evaluation::build(
                input.history,
                (
                    &list_plugin_eval_runs(pool, input.window, input.marketplace_id).await?,
                    &list_plugin_eval_tools(pool, input.window, input.marketplace_id).await?,
                ),
                wanted,
            )),
            ..TabViews::default()
        },
        Tab::Compare => TabViews {
            compare: Some(
                compare::build(
                    pool,
                    input.window,
                    compare::Recorded {
                        rows: input.rows,
                        scores: input.scores,
                    },
                    wanted,
                )
                .await?,
            ),
            ..TabViews::default()
        },
        Tab::Distribution => TabViews {
            distribution: Some(distribution::build(pool, managed, input.rows).await?),
            ..TabViews::default()
        },
    })
}
