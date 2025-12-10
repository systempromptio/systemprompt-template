use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

pub type BrowserRedirectHandler = Arc<
    dyn Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
        + Send
        + Sync,
>;

#[derive(Debug, Clone, Copy)]
pub struct RedirectMiddleware;

impl RedirectMiddleware {
    pub fn create_browser_redirect_handler(
        browser_redirect_service: Arc<systemprompt_core_oauth::services::BrowserRedirectService>,
        server_base_url: String,
    ) -> BrowserRedirectHandler {
        Arc::new(move |original_url: &str| {
            let service = browser_redirect_service.clone();
            let server_base_url = server_base_url.clone();
            let original_url = original_url.to_string();
            Box::pin(async move {
                match service
                    .create_oauth_redirect(&original_url, &server_base_url)
                    .await
                {
                    Ok(response) => response.into_response(),
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            })
        })
    }
}
