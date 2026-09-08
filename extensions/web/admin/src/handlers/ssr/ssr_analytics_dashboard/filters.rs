//! Filter-bar assembly for the dashboard: the group and project selects and the
//! day/week bucket toggle, plus the hidden fields the GET form must carry.
//! Split from `mod.rs` at the 300-line ceiling.

use crate::handlers::ssr::list_view::scope_filter_view;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;

use sqlx::PgPool;

use super::context::{DashboardTab, FiltersView};
use super::{AnalyticsDashboardQuery, BASE_URL, urls};

pub(super) async fn build_filters(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &AnalyticsDashboardQuery,
    request: &ScopeRequest,
    bucket: SeriesBucket,
) -> FiltersView {
    // Why: the filter form is a plain GET, so everything not expressed by its
    // select must ride along as hidden fields or submitting it would reset
    // the tab and window.
    let hidden = vec![
        (
            "tab".to_owned(),
            DashboardTab::from_query(query.tab.as_deref())
                .as_str()
                .to_owned(),
        ),
        (
            "preset".to_owned(),
            query.preset.clone().unwrap_or_default(),
        ),
        ("from".to_owned(), query.from.clone().unwrap_or_default()),
        ("to".to_owned(), query.to.clone().unwrap_or_default()),
        (
            "bucket".to_owned(),
            query.bucket.clone().unwrap_or_default(),
        ),
    ];

    FiltersView {
        scope: scope_filter_view(pool, user_ctx, request, BASE_URL, hidden).await,
        bucket_links: urls::bucket_links(query, bucket == SeriesBucket::Week),
    }
}
