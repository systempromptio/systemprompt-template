//! Header assertions for redirect and login contract tests.

use super::{App, Call};
use axum::http::StatusCode;
use tower::ServiceExt;

impl App {
    pub async fn response_headers(&self, call: Call<'_>) -> (StatusCode, ResponseHeaders) {
        self.response_headers_with(call, &[]).await
    }

    // Status plus the two response headers the redirect-driven flows are
    // specified in terms of.
    //
    // `call` reads the body, which is empty on a redirect: for the SSO flows
    // the entire outcome — where the browser goes, and what state it carries
    // there — lives in `Location` and `Set-Cookie`.
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
        let set_cookie = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(ToOwned::to_owned)
            .collect();
        (
            status,
            ResponseHeaders {
                location,
                set_cookie,
            },
        )
    }
}

// Headers carrying the outcome of a browser login or redirect.
pub struct ResponseHeaders {
    pub location: Option<String>,
    pub set_cookie: Vec<String>,
}
