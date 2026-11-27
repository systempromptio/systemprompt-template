//! Header assertions for redirect and login contract tests.

use super::{App, Call};
use axum::http::StatusCode;
use tower::ServiceExt;

impl App {
    pub async fn response_headers(&self, call: Call<'_>) -> (StatusCode, ResponseHeaders) {
        self.response_headers_with(call, &[]).await
    }

    // Status plus the `Location` a redirect-driven flow is specified in terms
    // of: `call` reads the body, which is empty on a redirect.
    pub async fn response_headers_with(
        &self,
        call: Call<'_>,
        extra_headers: &[(&str, &str)],
    ) -> (StatusCode, ResponseHeaders) {
        let request = self.build_request(call, None, extra_headers);
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let status = response.status();
        let headers = response.headers();
        let location = headers
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        (status, ResponseHeaders { location })
    }
}

// Headers carrying the outcome of a browser login or redirect.
pub struct ResponseHeaders {
    pub location: Option<String>,
}
