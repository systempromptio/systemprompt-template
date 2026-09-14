//! Sidebar marketplace-count context injected into every admin page.
//!
//! `marketplace_context_middleware` attaches a [`MarketplaceContext`] to each
//! request so the layout can show plugin / skill / MCP counts. The counts are
//! resolved from `services/` on disk and cached process-wide for
//! `MARKETPLACE_CACHE_TTL` to keep the filesystem walk off the hot path.

use super::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;

pub(crate) async fn marketplace_context_middleware(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let result = async {
        let path = crate::handlers::shared::get_services_path()?;
        let plugins = crate::repositories::marketplace::plugins::list_plugins_for_user(
            &pool,
            &path,
            &user_ctx.user_id,
        )
        .await?;
        Ok::<_, crate::error::AdminError>(
            crate::repositories::marketplace::plugins::count_visible_items(&plugins),
        )
    }
    .await;
    let counts = match result {
        Ok(counts) => counts,
        Err(error) => {
            tracing::error!(%error, user_id = %user_ctx.user_id, "catalog authorization unavailable");
            return crate::error::AdminError::Unavailable(
                "Catalog authorization unavailable".into(),
            )
            .into_response();
        },
    };
    let site_url = systemprompt::models::Config::get().map_or_else(
        |_| String::new(),
        |c| c.api_external_url.trim_end_matches('/').to_owned(),
    );

    let ctx = MarketplaceContext {
        user_id: user_ctx.user_id.clone(),
        site_url,
        total_plugins: counts.total_plugins,
        total_skills: counts.total_skills,
        agents_count: counts.agents_count,
        mcp_count: counts.mcp_count,
        rank_level: 1,
        rank_name: String::from("Beginner"),
        rank_tier: systemprompt_web_shared::RankTier::Bronze,
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::from("Apprentice"),
        xp_to_next_rank: 100,
    };

    request.extensions_mut().insert(ctx);
    next.run(request).await
}
