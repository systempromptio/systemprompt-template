use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use sqlx::PgPool;
use tokio::sync::RwLock;

use super::handlers::extract_user_from_cookie;
use super::repositories::plugins::MarketplaceCounts;
use super::types::{MarketplaceContext, UserContext};

struct CachedMarketplace {
    counts: MarketplaceCounts,
    site_url: String,
    fetched_at: Instant,
}

static MARKETPLACE_CACHE: LazyLock<RwLock<Option<CachedMarketplace>>> =
    LazyLock::new(|| RwLock::new(None));
const MARKETPLACE_CACHE_TTL: Duration = Duration::from_secs(300);

pub async fn user_context_middleware(
    State(pool): State<Arc<PgPool>>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let session = match extract_user_from_cookie(headers) {
        Ok(s) => s,
        Err(reason) => {
            tracing::warn!(reason = %reason, "UserContext middleware: no valid session");
            return next.run(request).await;
        }
    };

    let (roles, department) = fetch_user_roles_department(&pool, session.user_id.as_str())
        .await
        .unwrap_or_else(|| (vec!["user".to_string()], String::new()));

    let is_admin = roles.contains(&"admin".to_string());
    let ctx = UserContext {
        user_id: session.user_id,
        username: session.username,
        email: session.email,
        roles,
        department,
        is_admin,
        email_verified: false,
    };

    request.extensions_mut().insert(ctx);
    next.run(request).await
}

async fn fetch_user_roles_department(
    pool: &PgPool,
    user_id: &str,
) -> Option<(Vec<String>, String)> {
    #[derive(sqlx::FromRow)]
    struct UserRoleRow {
        roles: Vec<String>,
        department: Option<String>,
    }

    sqlx::query_as::<_, UserRoleRow>("SELECT roles, department FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user roles");
        })
        .ok()
        .flatten()
        .map(|row| (row.roles, row.department.unwrap_or_else(String::new)))
}

pub async fn marketplace_context_middleware(
    Extension(user_ctx): Extension<UserContext>,
    mut request: Request,
    next: Next,
) -> Response {
    use super::repositories::plugin_jwt::generate_plugin_token;

    let (counts, site_url) = get_cached_marketplace(&user_ctx.roles).await;

    let ctx = MarketplaceContext {
        user_id: user_ctx.user_id.to_string(),
        site_url,
        total_plugins: counts.total_plugins,
        total_skills: counts.total_skills,
        agents_count: counts.agents_count,
        mcp_count: counts.mcp_count,
        tier_name: String::from("Free"),
        is_premium: false,
        rank_level: 1,
        rank_name: String::from("Beginner"),
        rank_tier: String::from("bronze"),
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::from("Apprentice"),
        xp_to_next_rank: 100,
        plugin_token: generate_plugin_token(
            &user_ctx.user_id,
            user_ctx.email.as_ref(),
            "cowork-bundle",
        )
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to generate plugin token");
            String::new()
        }),
    };

    request.extensions_mut().insert(ctx);
    next.run(request).await
}

async fn get_cached_marketplace(roles: &[String]) -> (MarketplaceCounts, String) {
    use super::repositories;
    use systemprompt::models::{Config, ProfileBootstrap};

    {
        let cache = MARKETPLACE_CACHE.read().await;
        if let Some(ref cached) = *cache {
            if cached.fetched_at.elapsed() < MARKETPLACE_CACHE_TTL {
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
    }

    let roles = roles.to_vec();
    let (counts, site_url) = tokio::task::spawn_blocking(move || {
        let site_url = Config::get().map_or_else(
            |_| String::new(),
            |c| c.api_external_url.trim_end_matches('/').to_string(),
        );

        let counts = ProfileBootstrap::get()
            .map(|p| std::path::PathBuf::from(&p.paths.services))
            .map_err(|e| {
                tracing::warn!(error = %e, "Failed to get profile bootstrap for marketplace counts");
            })
            .ok()
            .and_then(|p| {
                repositories::count_marketplace_items(&p, &roles)
                    .map_err(|e| {
                        tracing::warn!(error = %e, "Failed to count marketplace items");
                    })
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
    });

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

pub async fn require_user_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => {
            let uri = request.uri().path().to_string();
            let redirect_url = format!("/admin/login?redirect={uri}");
            axum::response::Redirect::temporary(&redirect_url).into_response()
        }
    }
}

pub async fn require_auth_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response(),
    }
}

pub async fn require_admin_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if ctx.is_admin => next.run(request).await,
        _ => (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response(),
    }
}

pub async fn auth_me_handler(Extension(user_ctx): Extension<UserContext>) -> Response {
    if user_ctx.user_id.as_str().is_empty() {
        return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response();
    }
    axum::Json(serde_json::json!({
        "user_id": user_ctx.user_id,
        "username": user_ctx.username,
        "email": user_ctx.email,
        "roles": user_ctx.roles,
        "department": user_ctx.department,
        "is_admin": user_ctx.is_admin,
    }))
    .into_response()
}
