use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use axum::Extension;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};
use tokio::sync::RwLock;

use super::handlers::extract_user_from_cookie;
use super::handlers::shared::ErrorBody;
use super::repositories::plugins::MarketplaceCounts;
use super::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct AuthMeResponse {
    user_id: UserId,
    username: String,
    email: Email,
    roles: Vec<String>,
    department: String,
    is_admin: bool,
}

struct CachedMarketplace {
    counts: MarketplaceCounts,
    site_url: String,
    fetched_at: Instant,
}

static MARKETPLACE_CACHE: LazyLock<RwLock<Option<CachedMarketplace>>> =
    LazyLock::new(|| RwLock::new(None));
const MARKETPLACE_CACHE_TTL: Duration = Duration::from_mins(5);

pub(crate) async fn user_context_middleware(
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
        },
    };

    let (roles, department) = fetch_user_roles_department(&pool, &session.user_id)
        .await
        .unwrap_or_else(|| (vec!["user".to_owned()], String::new()));

    let is_admin = roles.contains(&"admin".to_owned());
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
    user_id: &UserId,
) -> Option<(Vec<String>, String)> {
    super::repositories::get_user_roles_department(pool, user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user roles");
        })
        .ok()
        .flatten()
}

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
    })
}

pub(crate) async fn require_user_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => {
            let uri = request
                .extensions()
                .get::<axum::extract::OriginalUri>()
                .map_or_else(
                    || request.uri().path().to_owned(),
                    |o| o.0.path().to_owned(),
                );
            let redirect_url = format!("/admin/login?redirect={uri}");
            axum::response::Redirect::temporary(&redirect_url).into_response()
        },
    }
}

pub(crate) async fn require_auth_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(ErrorBody {
                error: "Authentication required".to_owned(),
            }),
        )
            .into_response(),
    }
}

pub(crate) async fn require_admin_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if ctx.is_admin => next.run(request).await,
        _ => (
            StatusCode::FORBIDDEN,
            axum::Json(ErrorBody {
                error: "Admin access required".to_owned(),
            }),
        )
            .into_response(),
    }
}

/// Restrict non-admin users to the profile page, settings page, and a few
/// account-management endpoints. Other admin routes redirect to /admin/profile.
///
/// Admins pass through unchanged. Anonymous users are handled by
/// `require_user_middleware` which runs after this layer.
pub(crate) async fn non_admin_gate_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let user_ctx = request.extensions().get::<UserContext>().cloned();

    let Some(ctx) = user_ctx else {
        return next.run(request).await;
    };
    if ctx.is_admin || ctx.user_id.as_str().is_empty() {
        return next.run(request).await;
    }

    if is_non_admin_allowed_path(path) {
        next.run(request).await
    } else {
        axum::response::Redirect::to("/admin/profile").into_response()
    }
}

fn is_non_admin_allowed_path(path: &str) -> bool {
    path.starts_with("/admin/profile")
        || path.starts_with("/admin/settings")
        || path.starts_with("/admin/auth/")
        || path.starts_with("/admin/api/")
        || path == "/admin/logout"
        || path == "/admin/login"
        || path == "/admin/register"
        || path == "/admin/add-passkey"
        || path == "/admin/verify-pending"
        || path == "/admin/setup"
        || path == "/admin/demo-register"
        || path == "/admin/"
        || path == "/admin"
}

pub(crate) async fn auth_me_handler(Extension(user_ctx): Extension<UserContext>) -> Response {
    if user_ctx.user_id.as_str().is_empty() {
        return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response();
    }
    axum::Json(AuthMeResponse {
        user_id: user_ctx.user_id,
        username: user_ctx.username,
        email: user_ctx.email,
        roles: user_ctx.roles,
        department: user_ctx.department,
        is_admin: user_ctx.is_admin,
    })
    .into_response()
}
