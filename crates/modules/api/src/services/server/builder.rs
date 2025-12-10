use anyhow::Result;
use axum::extract::DefaultBodyLimit;
use axum::routing::get;
use axum::{Json, Router};

use std::sync::Arc;
use std::time::Duration;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::AppContext;
use systemprompt_models::api::SingleResponse;
use systemprompt_models::{Config, SystemPaths};

use crate::models::server::ServerConfig;
use crate::services::middleware::{
    remove_trailing_slash, AnalyticsMiddleware, ContextMiddleware, CorsMiddleware,
    JwtContextExtractor, RouterExt, SessionMiddleware,
};
use crate::services::static_content::{
    serve_vite_app, smart_fallback_handler, StaticContentMatcher, StaticContentState,
};
use serde_json::json;
use systemprompt_core_database::DatabaseQuery;

const HEALTH_CHECK_QUERY: DatabaseQuery = DatabaseQuery::new("SELECT 1");

#[derive(Debug)]
pub struct ApiServer {
    router: Router,
    _config: ServerConfig,
}

impl ApiServer {
    pub fn new(router: Router) -> Self {
        Self::with_config(router, ServerConfig::default())
    }

    pub const fn with_config(router: Router, config: ServerConfig) -> Self {
        Self {
            router,
            _config: config,
        }
    }

    pub async fn serve(self, addr: &str) -> Result<()> {
        CliService::info(&format!("Attempting to bind to: {addr}"));
        let listener = self.create_listener(addr).await?;
        CliService::success(&format!("Successfully bound to {addr}"));
        CliService::info(&format!("Server is now listening on http://{addr}"));
        CliService::info(&format!(
            "Single instance running - process ID: {}",
            std::process::id()
        ));

        axum::serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await?;
        Ok(())
    }

    async fn create_listener(&self, addr: &str) -> Result<tokio::net::TcpListener> {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                self.handle_port_conflict(addr).await
            },
            Err(e) => Err(e.into()),
        }
    }

    async fn handle_port_conflict(&self, addr: &str) -> Result<tokio::net::TcpListener> {
        let port: u16 = addr
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse port from address: {addr}"))?;

        let kill_cmd = format!("lsof -ti:{port} | xargs -r kill -9 2>/dev/null || true");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&kill_cmd)
            .output()
            .ok();

        tokio::time::sleep(Duration::from_millis(500)).await;

        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => Ok(listener),
            Err(e) => Err(e.into()),
        }
    }
}

pub async fn setup_api_server(ctx: &AppContext) -> Result<ApiServer> {
    let mut router = Router::new();

    let rate_config = &ctx.config().rate_limits;

    if rate_config.disabled {
        CliService::warning("RATE LIMITING DISABLED - Only use in development/testing!");
    } else {
        CliService::section("Rate Limiting Active");
        CliService::info(&format!(
            "  OAuth Public: {}/sec",
            rate_config.oauth_public_per_second
        ));
        CliService::info(&format!(
            "  OAuth Auth: {}/sec",
            rate_config.oauth_auth_per_second
        ));
        CliService::info(&format!(
            "  Contexts: {}/sec",
            rate_config.contexts_per_second
        ));
        CliService::info(&format!("  Tasks: {}/sec", rate_config.tasks_per_second));
        CliService::info(&format!(
            "  Artifacts: {}/sec",
            rate_config.artifacts_per_second
        ));
        CliService::info(&format!(
            "  Agent Registry: {}/sec",
            rate_config.agent_registry_per_second
        ));
        CliService::info(&format!("  Agents: {}/sec", rate_config.agents_per_second));
        CliService::info(&format!(
            "  MCP Registry: {}/sec",
            rate_config.mcp_registry_per_second
        ));
        CliService::info(&format!("  MCP: {}/sec", rate_config.mcp_per_second));
        CliService::info(&format!("  Stream: {}/sec", rate_config.stream_per_second));
        CliService::info(&format!(
            "  Content: {}/sec",
            rate_config.content_per_second
        ));
        CliService::info(&format!(
            "  Burst Multiplier: {}x",
            rate_config.burst_multiplier
        ));
    }

    let jwt_extractor =
        JwtContextExtractor::new(ctx.config().jwt_secret.clone(), ctx.db_pool().clone());

    let public_middleware = ContextMiddleware::public(jwt_extractor.clone());
    let user_middleware = ContextMiddleware::user_only(jwt_extractor.clone());
    let full_middleware = ContextMiddleware::full(jwt_extractor.clone());
    let mcp_middleware = ContextMiddleware::mcp(jwt_extractor.clone());

    // OAuth endpoints - Split into public and authenticated routes
    // Public routes (anon token generation): 2/sec per IP (120/min) - prevent flood
    // attacks
    router = router.nest(
        "/api/v1/core/oauth",
        systemprompt_core_oauth::api::public_router(ctx)
            .with_rate_limit(rate_config, rate_config.oauth_public_per_second)
            .with_auth_middleware(public_middleware.clone()),
    );

    // Authenticated OAuth routes (WebAuthn, etc.): 2/sec per IP (120/min) - prevent
    // abuse
    router = router.nest(
        "/api/v1/core/oauth",
        systemprompt_core_oauth::api::authenticated_router(ctx)
            .with_rate_limit(rate_config, rate_config.oauth_auth_per_second)
            .with_auth_middleware(user_middleware.clone()),
    );

    // Core API endpoints - MODERATE rate limiting (standard CRUD operations)
    // Contexts: 50/sec per IP (3000/min) - conversation management, artifact
    // streaming
    router = router.nest(
        "/api/v1/core/contexts",
        systemprompt_core_agent::api::contexts_router()
            .with_state(ctx.clone())
            .with_rate_limit(rate_config, rate_config.contexts_per_second)
            .with_auth_middleware(user_middleware.clone()),
    );

    // Internal webhook endpoint for MCP servers (requires JWT authentication)
    router = router.nest(
        "/api/v1/webhook",
        systemprompt_core_agent::api::webhook_router()
            .with_state(ctx.clone())
            .with_auth_middleware(user_middleware.clone()),
    );

    // Tasks: 10/sec per IP (600/min) - task operations
    router = router.nest(
        "/api/v1/core/tasks",
        systemprompt_core_agent::api::tasks_router()
            .with_state(ctx.clone())
            .with_rate_limit(rate_config, rate_config.tasks_per_second)
            .with_auth_middleware(user_middleware.clone()),
    );

    // Artifacts: 15/sec per IP (900/min) - higher limit for artifact retrieval
    router = router.nest(
        "/api/v1/core/artifacts",
        systemprompt_core_agent::api::artifacts_router()
            .with_state(ctx.clone())
            .with_rate_limit(rate_config, rate_config.artifacts_per_second)
            .with_auth_middleware(user_middleware.clone()),
    );

    // Agent registry - RELAXED (read-only, public)
    // Registry: 20/sec per IP (1200/min) - agent discovery
    router = router.nest(
        "/api/v1/agents/registry",
        systemprompt_core_agent::api::registry_router(ctx)
            .with_rate_limit(rate_config, rate_config.agent_registry_per_second)
            .with_auth_middleware(public_middleware.clone()),
    );

    // Agent proxy - STRICT (expensive AI operations)
    // Agents: 3/sec per IP (180/min) - A2A protocol interactions
    router = router.nest(
        "/api/v1/agents",
        crate::api::routes::proxy::agents::router(ctx)
            .with_rate_limit(rate_config, rate_config.agents_per_second)
            .with_auth_middleware(full_middleware.clone()),
    );

    // MCP registry - RELAXED (read-only, public)
    // Registry: 20/sec per IP (1200/min) - server discovery
    router = router.nest(
        "/api/v1/mcp/registry",
        systemprompt_core_mcp::api::registry_router(ctx)
            .with_rate_limit(rate_config, rate_config.mcp_registry_per_second)
            .with_auth_middleware(public_middleware.clone()),
    );

    // MCP proxy - RELAXED (tool execution)
    // MCP: 100/sec per IP (6000/min) - tool calls, SSE connections, protocol
    // operations
    router = router.nest(
        "/api/v1/mcp",
        crate::api::routes::proxy::mcp::router(ctx)
            .with_rate_limit(rate_config, rate_config.mcp_per_second)
            .with_auth_middleware(mcp_middleware.clone()),
    );

    // Stream endpoints - STRICT (persistent connections)
    // Streams: 1/sec per IP (60/min) - SSE connection establishment
    router = router.nest(
        "/api/v1/stream",
        crate::services::routes::stream::stream_router(ctx)
            .with_rate_limit(rate_config, rate_config.stream_per_second)
            .with_auth_middleware(user_middleware.clone()),
    );

    // Blog/Content API - RELAXED (read-only, public)
    // Content: 20/sec per IP (1200/min) - blog posts, pages, content queries
    router = router.nest(
        "/api/v1/content",
        systemprompt_core_blog::api::router(ctx)
            .with_rate_limit(rate_config, rate_config.content_per_second)
            .with_auth_middleware(public_middleware.clone()),
    );

    // Link redirect handler at root level - RELAXED (public, read-only with click
    // tracking)
    router = router.merge(
        systemprompt_core_blog::api::redirect_router(ctx)
            .with_rate_limit(rate_config, rate_config.content_per_second)
            .with_auth_middleware(public_middleware.clone()),
    );

    let config = Config::global();
    let content_config_path = SystemPaths::content_config(config);
    let content_matcher = match StaticContentMatcher::from_config(
        content_config_path
            .to_str()
            .unwrap_or("crates/services/content/config.yml"),
    ) {
        Ok(matcher) => Arc::new(matcher),
        Err(e) => {
            CliService::warning(&format!("Failed to load content config: {e}"));
            CliService::info("   Static content matching will be disabled");
            Arc::new(StaticContentMatcher::empty())
        },
    };

    // HTML content served by Rust (with analytics)
    // Static assets (CSS, JS, images, fonts) served by nginx
    let static_state = StaticContentState {
        ctx: Arc::new(ctx.clone()),
        matcher: content_matcher,
        route_classifier: ctx.route_classifier().clone(),
    };

    let static_router = Router::new()
        .route("/", get(serve_vite_app))
        .fallback(smart_fallback_handler)
        .with_state(static_state)
        .with_auth_middleware(public_middleware.clone());

    router = router.merge(static_router);

    router = router.merge(discovery_router(ctx).with_auth_middleware(public_middleware.clone()));

    router = router.merge(wellknown_router(ctx).with_auth_middleware(public_middleware.clone()));

    router = apply_global_middleware(router, ctx).await?;

    Ok(ApiServer::new(router))
}

fn discovery_router(ctx: &AppContext) -> Router {
    Router::new()
        .route("/api/v1", get(handle_root_discovery))
        .route("/api/v1/health", get(handle_health))
        .with_state(ctx.clone())
}

fn wellknown_router(ctx: &AppContext) -> Router {
    systemprompt_core_oauth::api::wellknown::wellknown_routes(ctx)
        .merge(crate::api::routes::wellknown_router(ctx))
}

async fn apply_global_middleware(router: Router, ctx: &AppContext) -> Result<Router> {
    let mut router = router;

    router = router.layer(DefaultBodyLimit::max(100 * 1024 * 1024));

    // Apply analytics middleware (will run AFTER session and context middleware)
    let analytics_middleware = AnalyticsMiddleware::new(Arc::new(ctx.clone()));
    router = router.layer(axum::middleware::from_fn({
        let middleware = analytics_middleware;
        move |req, next| {
            let middleware = middleware.clone();
            async move { middleware.track_request(req, next).await }
        }
    }));

    // Apply global context middleware (extracts conversation context)
    // This runs AFTER session middleware (middleware wraps in reverse order)
    let jwt_extractor =
        JwtContextExtractor::new(ctx.config().jwt_secret.clone(), ctx.db_pool().clone());
    let global_context_middleware = ContextMiddleware::public(jwt_extractor);
    router = router.layer(axum::middleware::from_fn({
        let middleware = global_context_middleware;
        move |req, next| {
            let middleware = middleware.clone();
            async move { middleware.handle(req, next).await }
        }
    }));

    // Apply session middleware (creates/validates sessions for ALL routes)
    // This runs FIRST before all other middleware
    let session_middleware = SessionMiddleware::new(Arc::new(ctx.clone()));
    router = router.layer(axum::middleware::from_fn({
        let middleware = session_middleware;
        move |req, next| {
            let middleware = middleware.clone();
            async move { middleware.handle(req, next).await }
        }
    }));

    let cors = CorsMiddleware::build_layer()?;
    router = router.layer(cors);

    router = router.layer(axum::middleware::from_fn(remove_trailing_slash));

    Ok(router)
}

async fn handle_root_discovery(
    axum::extract::State(ctx): axum::extract::State<AppContext>,
) -> impl axum::response::IntoResponse {
    let base = &ctx.config().api_external_url;
    let data = json!({
        "name": format!("{} API", ctx.config().sitename),
        "version": "1.0.0",
        "description": "SystemPrompt OS API Gateway",
        "endpoints": {
            "health": format!("{}/api/v1/health", base),
            "oauth": {
                "href": format!("{}/api/v1/core/oauth", base),
                "description": "OAuth2/OIDC authentication and WebAuthn",
                "endpoints": {
                    "authorize": format!("{}/api/v1/core/oauth/authorize", base),
                    "token": format!("{}/api/v1/core/oauth/token", base),
                    "userinfo": format!("{}/api/v1/core/oauth/userinfo", base),
                    "introspect": format!("{}/api/v1/core/oauth/introspect", base),
                    "revoke": format!("{}/api/v1/core/oauth/revoke", base),
                    "webauthn": format!("{}/api/v1/core/oauth/webauthn", base)
                }
            },
            "core": {
                "href": format!("{}/api/v1/core", base),
                "description": "Core conversation, task, and artifact management",
                "endpoints": {
                    "contexts": format!("{}/api/v1/core/contexts", base),
                    "tasks": format!("{}/api/v1/core/tasks", base),
                    "artifacts": format!("{}/api/v1/core/artifacts", base)
                }
            },
            "agents": {
                "href": format!("{}/api/v1/agents/registry", base),
                "description": "A2A protocol agent registry and proxy",
                "endpoints": {
                    "registry": format!("{}/api/v1/agents/registry", base),
                    "proxy": format!("{}/api/v1/agents/{{agent_id}}", base)
                }
            },
            "mcp": {
                "href": format!("{}/api/v1/mcp/registry", base),
                "description": "MCP server registry and lifecycle management",
                "endpoints": {
                    "registry": format!("{}/api/v1/mcp/registry", base),
                    "proxy": format!("{}/api/v1/mcp/{{server_name}}", base)
                }
            },
            "stream": {
                "href": format!("{}/api/v1/stream", base),
                "description": "Server-Sent Events (SSE) for real-time updates",
                "endpoints": {
                    "contexts": format!("{}/api/v1/stream/contexts", base)
                }
            }
        },
        "wellknown": {
            "oauth": format!("{}/.well-known/oauth-authorization-server", base),
            "agent": format!("{}/.well-known/agent-card.json", base)
        }
    });

    Json(SingleResponse::new(data))
}

async fn handle_health(
    axum::extract::State(ctx): axum::extract::State<AppContext>,
) -> impl axum::response::IntoResponse {
    use std::path::Path;
    use systemprompt_core_database::DatabaseProvider;
    use systemprompt_models::repository::ServiceRepository;

    let db_status = match ctx.db_pool().fetch_optional(&HEALTH_CHECK_QUERY, &[]).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let service_repo = ServiceRepository::new(ctx.db_pool().clone());

    let (agent_count, agent_status) = match service_repo.count_running_services("agent").await {
        Ok(count) if count > 0 => (count, "healthy"),
        Ok(_) => (0, "no_agents"),
        Err(_) => (0, "error"),
    };

    let (mcp_count, mcp_status) = match service_repo.count_running_services("mcp").await {
        Ok(count) if count > 0 => (count, "healthy"),
        Ok(_) => (0, "no_servers"),
        Err(_) => (0, "error"),
    };

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "/app/core/web/dist".to_string());
    let (sitemap_exists, sitemap_status) = if Path::new(&web_dir).join("sitemap.xml").exists() {
        (true, "present")
    } else {
        (false, "missing")
    };

    let (index_exists, index_status) = if Path::new(&web_dir).join("index.html").exists() {
        (true, "present")
    } else {
        (false, "missing")
    };

    let overall_status = if db_status == "healthy"
        && agent_status != "error"
        && mcp_status != "error"
        && sitemap_exists
        && index_exists
    {
        "healthy"
    } else {
        "degraded"
    };

    let data = json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "database": db_status,
        "services": {
            "agents": {
                "status": agent_status,
                "running": agent_count
            },
            "mcp": {
                "status": mcp_status,
                "running": mcp_count
            }
        },
        "content": {
            "sitemap": sitemap_status,
            "index": index_status
        }
    });

    Json(SingleResponse::new(data))
}
