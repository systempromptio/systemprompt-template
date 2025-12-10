use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use serde_json::json;
use std::sync::Arc;

use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{AnalyticsEvent, AnalyticsRepository, LogLevel, LogService};
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::services::ScannerDetector;
use systemprompt_core_system::{AppContext, RequestContext};
use systemprompt_identifiers::SessionId;
use systemprompt_models::RouteClassifier;

#[derive(Debug, Clone)]
pub struct AnalyticsMiddleware {
    session_repo: Arc<AnalyticsSessionRepository>,
    analytics_repo: Arc<AnalyticsRepository>,
    db_pool: DbPool,
    route_classifier: Arc<RouteClassifier>,
}

impl AnalyticsMiddleware {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let db_pool = app_context.db_pool().clone();
        let session_repo = Arc::new(AnalyticsSessionRepository::new(db_pool.clone()));
        let analytics_repo = Arc::new(AnalyticsRepository::new(db_pool.clone()));
        let route_classifier = app_context.route_classifier().clone();

        Self {
            session_repo,
            analytics_repo,
            db_pool,
            route_classifier,
        }
    }

    fn sanitize_uri(uri: &http::Uri) -> String {
        let path = uri.path();

        if let Some(query) = uri.query() {
            let sanitized_params: Vec<String> = query
                .split('&')
                .map(|param| {
                    if let Some((key, value)) = param.split_once('=') {
                        let key_lower = key.to_lowercase();
                        if key_lower == "token"
                            || key_lower == "password"
                            || key_lower == "api_key"
                            || key_lower == "apikey"
                            || key_lower == "secret"
                            || key_lower == "authorization"
                            || key_lower == "auth"
                        {
                            format!("{key}=[REDACTED]")
                        } else {
                            format!("{key}={value}")
                        }
                    } else {
                        param.to_string()
                    }
                })
                .collect();

            format!("{path}?{}", sanitized_params.join("&"))
        } else {
            path.to_string()
        }
    }

    pub async fn track_request(
        &self,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let method = request.method().clone();
        let uri = request.uri().clone();

        let Some(req_ctx) = request.extensions().get::<RequestContext>().cloned() else {
            return Ok(next.run(request).await);
        };

        if !req_ctx.request.is_tracked {
            return Ok(next.run(request).await);
        }

        // Extract headers before request is consumed
        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        let referer = request
            .headers()
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        let start_time = std::time::Instant::now();
        let response = next.run(request).await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        let status_code = response.status();

        let should_track = self
            .route_classifier
            .should_track_analytics(uri.path(), method.as_str());

        // Check if this is a scanner/malicious request
        let is_scanner =
            ScannerDetector::is_scanner(Some(uri.path()), user_agent.as_deref(), None, None);

        if should_track {
            let endpoint = format!("{} {}", method, uri.path());
            let path = uri.path();
            let status_code_u16 = status_code.as_u16();

            // If this is a scanner request, mark the session as scanner
            if is_scanner {
                self.spawn_mark_scanner_task(req_ctx.request.session_id.clone());
            }

            self.spawn_session_tracking_task(req_ctx.request.session_id.clone());

            self.spawn_analytics_event_task(
                req_ctx.clone(),
                endpoint,
                path.to_string(),
                method.to_string(),
                uri.clone(),
                status_code_u16,
                response_time_ms,
                user_agent,
                referer,
            );
        }

        Ok(response)
    }

    fn spawn_session_tracking_task(&self, session_id: SessionId) {
        let session_repo = self.session_repo.clone();
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            let logger = LogService::system(db_pool.clone());

            if let Err(e) = session_repo.update_activity(session_id.as_str()).await {
                logger
                    .error(
                        "analytics",
                        &format!("Failed to update session activity: {e}"),
                    )
                    .await
                    .ok();
            }

            if let Err(e) = session_repo
                .increment_request_count(session_id.as_str())
                .await
            {
                logger
                    .error(
                        "analytics",
                        &format!("Failed to increment request count: {e}"),
                    )
                    .await
                    .ok();
            }
        });
    }

    fn spawn_mark_scanner_task(&self, session_id: SessionId) {
        let session_repo = self.session_repo.clone();

        tokio::spawn(async move {
            let _ = session_repo.mark_as_scanner(session_id.as_str()).await;
        });
    }

    #[allow(clippy::needless_pass_by_value)]
    fn spawn_analytics_event_task(
        &self,
        req_ctx: RequestContext,
        endpoint: String,
        path: String,
        method: String,
        uri: http::Uri,
        status_code: u16,
        response_time_ms: u64,
        user_agent: Option<String>,
        referer: Option<String>,
    ) {
        let db_pool = self.db_pool.clone();
        let analytics_repo = self.analytics_repo.clone();
        let sanitized_uri = Self::sanitize_uri(&uri);
        let route_classifier = self.route_classifier.clone();

        tokio::spawn(async move {
            let logger = LogService::new(db_pool.clone(), req_ctx.log_context());

            let message = format!("HTTP {status_code} - {method} {sanitized_uri}");
            let metadata = json!({
                "status_code": status_code,
                "method": method,
                "uri": sanitized_uri,
                "endpoint": endpoint,
                "trace_id": req_ctx.trace_id(),
                "user_agent": user_agent,
                "referer": referer
            });

            let event_metadata = route_classifier.get_event_metadata(&path, &method);

            let severity = if status_code >= 500 {
                "error"
            } else if status_code >= 400 {
                "warning"
            } else {
                "info"
            };

            let event = AnalyticsEvent {
                user_id: req_ctx.auth.user_id.clone(),
                session_id: req_ctx.request.session_id.clone(),
                context_id: req_ctx.execution.context_id.clone(),
                event_type: event_metadata.event_type.to_string(),
                event_category: event_metadata.event_category.to_string(),
                severity: severity.to_string(),
                endpoint: Some(endpoint),
                error_code: if status_code >= 400 {
                    Some(i32::from(status_code))
                } else {
                    None
                },
                response_time_ms: Some(response_time_ms as i32),
                agent_id: None,
                task_id: req_ctx.task_id().cloned(),
                message: Some(message.clone()),
                metadata: metadata.clone(),
            };

            if let Err(e) = analytics_repo.log_event(&event).await {
                logger
                    .error(
                        "analytics",
                        &format!("Failed to log analytics event: {e}"),
                    )
                    .await
                    .ok();
            }

            // Only log 5xx errors to console (4xx are tracked in analytics but don't need
            // console spam)
            if status_code >= 500 {
                logger
                    .log(
                        LogLevel::Error,
                        event_metadata.log_module,
                        &message,
                        Some(metadata),
                    )
                    .await
                    .ok();
            }
        });
    }
}
