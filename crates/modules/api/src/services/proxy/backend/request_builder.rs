use super::error::ProxyError;
use super::headers::HeaderInjector;
use super::url_resolver::UrlResolver;
use axum::body::{to_bytes, Body};
use axum::http::{HeaderMap, StatusCode};
use reqwest::Method;
use std::str::FromStr;
use systemprompt_core_system::RequestContext;
use systemprompt_models::repository::ServiceConfig;

#[derive(Debug, Clone, Copy)]
pub struct RequestBuilder;

impl RequestBuilder {
    pub async fn extract_body(body: Body) -> Result<Vec<u8>, StatusCode> {
        const MAX_BODY_SIZE: usize = 100 * 1024 * 1024;

        match to_bytes(body, MAX_BODY_SIZE).await {
            Ok(bytes) => Ok(bytes.to_vec()),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("length limit") || error_msg.contains("too large") {
                    Err(StatusCode::PAYLOAD_TOO_LARGE)
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            },
        }
    }

    pub fn parse_method(method_str: &str) -> Result<Method, StatusCode> {
        Method::from_str(method_str).map_err(|_| StatusCode::BAD_REQUEST)
    }

    pub fn build_request(
        client: &reqwest::Client,
        method: Method,
        url: &str,
        headers: &HeaderMap,
        body: Vec<u8>,
    ) -> Result<reqwest::RequestBuilder, StatusCode> {
        let mut req_builder = client.request(method, url);

        req_builder = Self::add_headers(req_builder, headers);

        if !body.is_empty() {
            req_builder = req_builder.body(body);
        }

        Ok(req_builder)
    }

    fn add_headers(
        mut req_builder: reqwest::RequestBuilder,
        headers: &HeaderMap,
    ) -> reqwest::RequestBuilder {
        for (key, value) in headers {
            if let Ok(value_str) = value.to_str() {
                let key_str = key.as_str();

                if Self::should_skip_header(key_str) {
                    continue;
                }

                if key_str.eq_ignore_ascii_case("authorization") {
                    if Self::is_valid_auth_header(value_str) {
                        req_builder = req_builder.header(key_str, value_str);
                    }
                } else {
                    req_builder = req_builder.header(key_str, value_str);
                }
            }
        }
        req_builder
    }

    fn should_skip_header(header_name: &str) -> bool {
        let lower_name = header_name.to_lowercase();
        matches!(lower_name.as_str(), "host" | "x-mcp-proxy-auth")
    }

    fn is_valid_auth_header(value: &str) -> bool {
        value != "Bearer" && !value.trim().eq_ignore_ascii_case("bearer")
    }
}

#[derive(Debug)]
pub struct ProxyRequestBuilder {
    service: ServiceConfig,
    path: String,
    method: Method,
    headers: HeaderMap,
    body: Vec<u8>,
    query: Option<String>,
    context: Option<RequestContext>,
}

impl ProxyRequestBuilder {
    pub fn new(service: ServiceConfig, path: &str, method: Method) -> Self {
        Self {
            service,
            path: path.to_string(),
            method,
            headers: HeaderMap::new(),
            body: Vec::new(),
            query: None,
            context: None,
        }
    }

    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_query(mut self, query: Option<&str>) -> Self {
        self.query = query.map(ToString::to_string);
        self
    }

    pub fn with_context(mut self, context: Option<RequestContext>) -> Self {
        self.context = context;
        self
    }

    pub fn build(
        mut self,
        client: &reqwest::Client,
    ) -> Result<reqwest::RequestBuilder, ProxyError> {
        if let Some(req_ctx) = &self.context {
            HeaderInjector::inject_context(&mut self.headers, req_ctx);
        }

        let url = self.build_url();

        let mut req_builder = client.request(self.method, &url);
        req_builder = Self::add_headers_to_request(req_builder, &self.headers);

        if !self.body.is_empty() {
            req_builder = req_builder.body(self.body);
        }

        Ok(req_builder)
    }

    fn build_url(&self) -> String {
        let base_url =
            UrlResolver::build_backend_url("http", "localhost", self.service.port, &self.path);

        UrlResolver::append_query_params(base_url, self.query.as_deref())
    }

    fn add_headers_to_request(
        mut req_builder: reqwest::RequestBuilder,
        headers: &HeaderMap,
    ) -> reqwest::RequestBuilder {
        for (key, value) in headers {
            if let Ok(value_str) = value.to_str() {
                let key_str = key.as_str();

                if Self::should_skip_header(key_str) {
                    continue;
                }

                if key_str.eq_ignore_ascii_case("authorization") {
                    if Self::is_valid_auth_header(value_str) {
                        req_builder = req_builder.header(key_str, value_str);
                    }
                } else {
                    req_builder = req_builder.header(key_str, value_str);
                }
            }
        }
        req_builder
    }

    fn should_skip_header(header_name: &str) -> bool {
        let lower_name = header_name.to_lowercase();
        matches!(lower_name.as_str(), "host" | "x-mcp-proxy-auth")
    }

    fn is_valid_auth_header(value: &str) -> bool {
        value != "Bearer" && !value.trim().eq_ignore_ascii_case("bearer")
    }
}
