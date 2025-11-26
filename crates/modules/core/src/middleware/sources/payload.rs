use axum::body::Body;
use axum::extract::Request;
use serde_json::Value;
use systemprompt_models::execution::ContextExtractionError;

#[derive(Debug, Clone, Copy)]
pub struct PayloadSource;

impl PayloadSource {
    pub async fn extract_context_id(body_bytes: &[u8]) -> Result<String, ContextExtractionError> {
        let payload: Value = serde_json::from_slice(body_bytes).map_err(|e| {
            ContextExtractionError::InvalidHeaderValue {
                header: "payload".to_string(),
                reason: format!("Invalid JSON: {e}"),
            }
        })?;

        payload
            .get("params")
            .and_then(|p| p.get("message"))
            .and_then(|m| m.get("contextId"))
            .and_then(|c| c.as_str())
            .map(ToString::to_string)
            .ok_or(ContextExtractionError::MissingContextId)
    }

    pub async fn read_and_reconstruct(
        request: Request<Body>,
    ) -> Result<(Vec<u8>, Request<Body>), ContextExtractionError> {
        let (parts, body) = request.into_parts();

        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                header: "body".to_string(),
                reason: format!("Failed to read body: {e}"),
            })?
            .to_vec();

        let new_body = Body::from(body_bytes.clone());
        let new_request = Request::from_parts(parts, new_body);

        Ok((body_bytes, new_request))
    }
}
