use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
};
use std::path::PathBuf;
use std::sync::Arc;

use super::config::StaticContentMatcher;
use systemprompt_core_blog::ContentRepository;
use systemprompt_core_system::AppContext;
use systemprompt_models::{RouteClassifier, RouteType};

#[derive(Clone, Debug)]
pub struct StaticContentState {
    pub ctx: Arc<AppContext>,
    pub matcher: Arc<StaticContentMatcher>,
    pub route_classifier: Arc<RouteClassifier>,
}

/// Serves HTML content with analytics tracking
/// Static assets (CSS, JS, images, fonts) are served by nginx
pub async fn serve_vite_app(
    State(state): State<StaticContentState>,
    uri: Uri,
    req_ctx: Option<axum::Extension<systemprompt_core_system::RequestContext>>,
) -> impl IntoResponse {
    let matcher = state.matcher;

    // Require explicit WEB_DIR configuration for security and clarity
    let dist_dir = match std::env::var("WEB_DIR") {
        Ok(web_dir) => PathBuf::from(web_dir),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::response::Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>SystemPrompt - Configuration Error</title>
    <style>
        body { font-family: sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; color: #d32f2f; }
        code { background: #ffebee; padding: 2px 6px; border-radius: 3px; font-weight: bold; }
    </style>
</head>
<body>
    <h1>Configuration Error</h1>
    <p>The web interface requires the <code>WEB_DIR</code> environment variable to be set.</p>
    <p>Set it to point to your built web directory, e.g.:</p>
    <pre>export WEB_DIR="/app/core/web/dist"</pre>
</body>
</html>
                "#)
            ).into_response();
        },
    };

    if !dist_dir.exists() || !dist_dir.join("index.html").exists() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>SystemPrompt - Build Missing</title>
    <style>
        body { font-family: sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; color: #d32f2f; }
    </style>
</head>
<body>
    <h1>Build Missing</h1>
    <p>Web assets not found at the configured WEB_DIR location.</p>
    <p>Build the web assets first.</p>
</body>
</html>
            "#)
        ).into_response();
    }

    let path = uri.path();

    // Handle static assets directly (JS, CSS, fonts, images)
    // In production, nginx serves these; in development, Rust must serve them
    if matches!(
        state.route_classifier.classify(path, "GET"),
        RouteType::StaticAsset { .. }
    ) {
        // For /generated/ paths, serve from storage directory
        // Maps /generated/images/... to storage/generated_images/...
        let asset_path = if path.starts_with("/generated/") {
            let storage_dir =
                std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".to_string());
            let relative_path = path
                .strip_prefix("/generated/")
                .unwrap_or("")
                .replace("images/", "generated_images/");
            PathBuf::from(&storage_dir).join(relative_path)
        } else {
            dist_dir.join(&path[1..])
        };

        if asset_path.exists() && asset_path.is_file() {
            match std::fs::read(&asset_path) {
                Ok(content) => {
                    let mime_type = match asset_path.extension().and_then(|ext| ext.to_str()) {
                        Some("js") => "application/javascript",
                        Some("css") => "text/css",
                        Some("woff") | Some("woff2") => "font/woff2",
                        Some("ttf") => "font/ttf",
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        Some("svg") => "image/svg+xml",
                        Some("ico") => "image/x-icon",
                        Some("json") => "application/json",
                        _ => "application/octet-stream",
                    };

                    return (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], content)
                        .into_response();
                },
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Error reading asset")
                        .into_response();
                },
            }
        } else {
            // Asset not found - return 404, not HTML fallback
            return (StatusCode::NOT_FOUND, "Asset not found").into_response();
        }
    }

    // Determine which HTML file to serve (static assets are handled above)
    // SPA (index.html) is ONLY served for root path "/"
    // All other routes must have prerendered HTML or return 404
    if path == "/" {
        // Root route - serve SPA
        let index_path = dist_dir.join("index.html");
        if index_path.exists() {
            match std::fs::read(&index_path) {
                Ok(content) => {
                    return (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/html")],
                        content,
                    )
                        .into_response();
                },
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error reading index.html",
                    )
                        .into_response();
                },
            }
        } else {
            return (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found").into_response();
        }
    }

    // Static text files
    if path == "/sitemap.xml" || path == "/robots.txt" || path == "/llms.txt" {
        let file_path = dist_dir.join(&path[1..]);
        if file_path.exists() {
            match std::fs::read(&file_path) {
                Ok(content) => {
                    let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
                        Some("xml") => "application/xml",
                        Some("txt") => "text/plain",
                        _ => "text/plain",
                    };
                    return (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], content)
                        .into_response();
                },
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Error reading file")
                        .into_response();
                },
            }
        } else {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    }

    // Parent route pages (e.g., /blog, /legal)
    // These are listing pages without slugs
    let parent_route_path = dist_dir.join(&path[1..]).join("index.html");
    if parent_route_path.exists() {
        match std::fs::read(&parent_route_path) {
            Ok(content) => {
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html")],
                    content,
                )
                    .into_response();
            },
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error reading parent route",
                )
                    .into_response();
            },
        }
    }

    // Content-matched paths (blog posts, pages, etc.)
    if let Some((slug, source_id)) = matcher.matches(path) {
        // Try to serve prerendered HTML file
        let exact_path = dist_dir.join(&path[1..]);
        if exact_path.exists() && exact_path.is_file() {
            return serve_html_with_analytics(
                exact_path,
                slug.clone(),
                source_id.clone(),
                req_ctx.as_ref().map(|ext| ext.0.clone()),
            )
            .await
            .into_response();
        }

        let index_path = dist_dir.join(&path[1..]).join("index.html");
        if index_path.exists() {
            return serve_html_with_analytics(
                index_path,
                slug.clone(),
                source_id.clone(),
                req_ctx.as_ref().map(|ext| ext.0.clone()),
            )
            .await
            .into_response();
        }

        // No prerendered file - check if content exists in database
        let content_repo = ContentRepository::new(state.ctx.db_pool().clone());
        match content_repo.get_by_slug(&slug).await {
            Ok(Some(_)) => {
                // Content exists but no prerendered HTML - this is a build issue
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::response::Html(format!(
                        r#"<!DOCTYPE html>
<html>
<head>
    <title>Content Not Prerendered</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; }}
        h1 {{ color: #d32f2f; }}
        code {{ background: #f5f5f5; padding: 2px 6px; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>Content Not Prerendered</h1>
    <p>The content exists in the database but has not been prerendered to HTML.</p>
    <p>Route: <code>{}</code></p>
    <p>Slug: <code>{}</code></p>
    <p>Run the prerendering build step to generate static HTML.</p>
</body>
</html>"#,
                        path, slug
                    ))
                ).into_response();
            },
            Ok(None) => {
                // Content doesn't exist - return 404
                return (
                    StatusCode::NOT_FOUND,
                    axum::response::Html(format!(
                        r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        a {{ color: #1976d2; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>404 - Page Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <p><a href="/">← Back to home</a></p>
</body>
</html>"#
                    ))
                ).into_response();
            },
            Err(e) => {
                // Database error
                tracing::error!("Database error checking content: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    .into_response();
            },
        }
    }

    // All other paths - 404
    (
        StatusCode::NOT_FOUND,
        axum::response::Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: system-ui, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; }
        h1 { color: #333; }
        a { color: #1976d2; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>404 - Page Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <p><a href="/">← Back to home</a></p>
</body>
</html>"#)
    ).into_response()
}

async fn serve_html_with_analytics(
    html_path: PathBuf,
    slug: String,
    source_id: String,
    req_ctx: Option<systemprompt_core_system::RequestContext>,
) -> impl IntoResponse {
    let html_content = match std::fs::read(&html_path) {
        Ok(content) => content,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error reading file").into_response();
        },
    };

    // Session already created by analytics middleware - just log for debugging
    if let Some(ctx) = req_ctx {
        if source_id == "blog" {
            let session_id = ctx.request.session_id.as_str();
            tracing::debug!(
                "Serving content with slug: {} (session: {})",
                slug,
                session_id
            );
        }
    }

    let mut response = (StatusCode::OK, html_content).into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, "text/html".parse().unwrap());

    response
}
