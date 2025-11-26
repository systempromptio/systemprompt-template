use anyhow::Result;
use axum::{extract::Request, http::StatusCode, routing::any, Router};

pub async fn create_proxy_router(target_host: &str, target_port: u16) -> Result<Router> {
    let target_url = format!("http://{target_host}:{target_port}");

    let router = Router::new().fallback(any(move |req: Request| {
        let url = target_url.clone();
        async move { forward_request(req, url).await }
    }));

    Ok(router)
}

async fn forward_request(
    req: Request,
    target_url: String,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let path = req.uri().path();
    let query = match req.uri().query() {
        Some(q) => format!("?{q}"),
        None => String::new(),
    };
    let full_url = format!("{target_url}{path}{query}");

    // Create HTTP client
    let client = reqwest::Client::new();

    // Build proxied request
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut proxied = client.request(method, &full_url);

    // Copy headers (except host)
    for (key, value) in req.headers() {
        if key != "host" {
            proxied = proxied.header(key.as_str(), value.as_bytes());
        }
    }

    // Get body
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !body_bytes.is_empty() {
        proxied = proxied.body(body_bytes.to_vec());
    }

    // Send request
    let response = proxied.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status_code = response.status().as_u16();
    let status = StatusCode::from_u16(status_code).map_err(|_| {
        eprintln!("Invalid status code from upstream: {status_code}");
        StatusCode::BAD_GATEWAY
    })?;

    let body = response
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok((status, body))
}

pub async fn create_load_balanced_proxy(targets: Vec<(String, u16)>) -> Result<Router> {
    // Simple round-robin load balancing
    let target_urls: Vec<String> = targets
        .iter()
        .map(|(host, port)| format!("http://{host}:{port}"))
        .collect();

    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let router = Router::new().fallback(any(move |req: Request| {
        let urls = target_urls.clone();
        let cnt = counter.clone();
        async move {
            let index = cnt.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % urls.len();
            let url = urls[index].clone();
            forward_request(req, url).await
        }
    }));

    Ok(router)
}
