//! SystemPrompt Template Server
//!
//! This demonstrates how to build a SystemPrompt application with extensions.

use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use systemprompt_blog_extension::{BlogConfig, BlogExtension};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    tracing::info!("Starting SystemPrompt Template Server");

    // Load configuration
    let config = load_config()?;

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/systemprompt".to_string());

    tracing::info!("Connecting to database...");
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?,
    );

    // Install extension schemas
    install_extension_schemas(&pool).await?;

    // Build and start the server
    start_server(pool, config).await?;

    Ok(())
}

/// Initialize tracing/logging.
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load application configuration.
fn load_config() -> Result<BlogConfig> {
    // Try to load from config file
    let config_path = std::env::var("BLOG_CONFIG")
        .unwrap_or_else(|_| "./services/config/blog.yaml".to_string());

    if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: BlogConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    } else {
        tracing::warn!("Blog config not found at {config_path}, using defaults");
        Ok(BlogConfig::default())
    }
}

/// Install database schemas for all extensions.
async fn install_extension_schemas(pool: &sqlx::PgPool) -> Result<()> {
    tracing::info!("Installing extension schemas");

    for (table, sql) in BlogExtension::schemas() {
        // Check if table exists
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;

        if !exists.0 {
            tracing::info!(table = %table, "Creating table");
            sqlx::raw_sql(sql).execute(pool).await?;
        } else {
            tracing::debug!(table = %table, "Table exists, skipping");
        }
    }

    tracing::info!("Extension schemas installed");
    Ok(())
}

/// Start the API server with extension routes mounted.
async fn start_server(pool: Arc<sqlx::PgPool>, config: BlogConfig) -> Result<()> {
    // Build core routes
    let mut app = Router::new().route("/health", axum::routing::get(|| async { "OK" }));

    // Mount blog extension routes
    let blog = BlogExtension::default();
    let blog_router = blog.router(pool.clone(), config);
    let base_path = BlogExtension::base_path();

    tracing::info!(extension = "blog", path = %base_path, "Mounting extension routes");
    app = app.nest(base_path, blog_router);

    // Mount redirect router at /r/
    let redirect_router = BlogExtension::redirect_router(pool);
    app = app.nest("/r", redirect_router);

    // Add middleware
    app = app
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    // Bind and serve
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!(%addr, "Starting server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
