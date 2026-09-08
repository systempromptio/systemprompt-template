//! Sidebar marketplace-count context injected into every admin page.
//!
//! `marketplace_context_middleware` attaches a [`MarketplaceContext`] to each
//! request so the layout can show plugin / skill / MCP counts. The counts are
//! resolved from `services/` on disk and cached process-wide for
//! `MARKETPLACE_CACHE_TTL` to keep the filesystem walk off the hot path.

use std::time::{Duration, Instant};

use axum::Extension;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::LazyLock;
use tokio::sync::RwLock;

use super::repositories::marketplace::plugins::MarketplaceCounts;
use super::types::{MarketplaceContext, UserContext};

struct CachedMarketplace {
    counts: MarketplaceCounts,
    site_url: String,
    fetched_at: Instant,
}

static MARKETPLACE_CACHE: LazyLock<RwLock<Option<CachedMarketplace>>> =
    LazyLock::new(|| RwLock::new(None));
const MARKETPLACE_CACHE_TTL: Duration = Duration::from_mins(5);

pub(crate) async fn marketplace_context_middleware(
    Extension(user_ctx): Extension<UserContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let (counts, site_url) = get_cached_marketplace(&user_ctx.roles).await;

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

async fn get_cached_marketplace(roles: &[String]) -> (MarketplaceCounts, String) {
    {
        let cache = MARKETPLACE_CACHE.read().await;
        if let Some(ref cached) = *cache
            && cached.fetched_at.elapsed() < MARKETPLACE_CACHE_TTL
        {
            return (
                MarketplaceCounts {
                    total_plugins: cached.counts.total_plugins,
                    total_skills: cached.counts.total_skills,
                    agents_count: cached.counts.agents_count,
                    mcp_count: cached.counts.mcp_count,
                },
                cached.site_url.clone(),
            );
        }
    }

    let (counts, site_url) = compute_marketplace_counts(roles.to_vec()).await;

    {
        let mut cache = MARKETPLACE_CACHE.write().await;
        *cache = Some(CachedMarketplace {
            counts,
            site_url: site_url.clone(),
            fetched_at: Instant::now(),
        });
    }

    (counts, site_url)
}

async fn compute_marketplace_counts(roles: Vec<String>) -> (MarketplaceCounts, String) {
    use super::repositories;
    use systemprompt::config::ProfileBootstrap;
    use systemprompt::models::Config;

    tokio::task::spawn_blocking(move || {
        let site_url = Config::get().map_or_else(
            |_| String::new(),
            |c| c.api_external_url.trim_end_matches('/').to_owned(),
        );

        let counts = ProfileBootstrap::get()
            .map(|p| std::path::PathBuf::from(&p.paths.services))
            .inspect_err(|e| tracing::warn!(error = %e, "Failed to get profile bootstrap for marketplace counts"))
            .ok()
            .and_then(|p| {
                repositories::marketplace::plugins::count_marketplace_items(&p, &roles)
                    .inspect_err(|e| tracing::warn!(error = %e, "Failed to count marketplace items"))
                    .ok()
            })
            .unwrap_or(MarketplaceCounts {
                total_plugins: 0,
                total_skills: 0,
                agents_count: 0,
                mcp_count: 0,
            });

        (counts, site_url)
    })
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "spawn_blocking for marketplace counts failed");
        (
            MarketplaceCounts {
                total_plugins: 0,
                total_skills: 0,
                agents_count: 0,
                mcp_count: 0,
            },
            String::new(),
        )
    })
}
