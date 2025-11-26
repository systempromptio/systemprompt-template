use axum::http::HeaderMap;
use systemprompt_models::execution::ContextExtractionError;

#[derive(Debug, Clone, Copy)]
pub struct HeaderSource;

impl HeaderSource {
    pub fn extract_required(
        headers: &HeaderMap,
        name: &str,
    ) -> Result<String, ContextExtractionError> {
        headers
            .get(name)
            .ok_or_else(|| ContextExtractionError::MissingHeader(name.to_string()))?
            .to_str()
            .map(ToString::to_string)
            .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                header: name.to_string(),
                reason: e.to_string(),
            })
    }

    pub fn extract_optional(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string)
    }
}
