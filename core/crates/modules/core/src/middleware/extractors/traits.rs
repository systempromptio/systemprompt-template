use async_trait::async_trait;
use axum::body::Body;
use axum::extract::Request;
use axum::http::HeaderMap;
use systemprompt_models::execution::{ContextExtractionError, RequestContext};

#[async_trait]
pub trait ContextExtractor: Send + Sync {
    async fn extract_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError>;

    async fn extract_from_request(
        &self,
        request: Request<Body>,
    ) -> Result<(RequestContext, Request<Body>), ContextExtractionError> {
        let headers = request.headers().clone();
        let context = self.extract_from_headers(&headers).await?;
        Ok((context, request))
    }

    async fn extract_user_only(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        self.extract_from_headers(headers).await
    }
}
