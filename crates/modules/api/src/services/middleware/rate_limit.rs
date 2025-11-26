use axum::Router;
use systemprompt_core_system::middleware::{ContextExtractor, ContextMiddleware};
use systemprompt_models::config::RateLimitConfig;
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub trait RouterExt<S> {
    fn with_rate_limit(self, rate_config: &RateLimitConfig, per_second: u64) -> Self;
    fn with_auth_middleware<E>(self, middleware: ContextMiddleware<E>) -> Self
    where
        E: ContextExtractor + Clone + Send + Sync + 'static;
}

impl<S> RouterExt<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_rate_limit(self, rate_config: &RateLimitConfig, per_second: u64) -> Self {
        if rate_config.disabled {
            return self;
        }

        // SmartIpKeyExtractor now works correctly because nginx's real_ip module
        // has already cleaned the X-Forwarded-For header to contain only the real client IP
        let rate_limit = tower_governor::governor::GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size((per_second * rate_config.burst_multiplier) as u32)
            .key_extractor(SmartIpKeyExtractor)
            .use_headers()
            .finish()
            .unwrap();

        self.layer(tower_governor::GovernorLayer::new(rate_limit))
    }

    fn with_auth_middleware<E>(self, middleware: ContextMiddleware<E>) -> Self
    where
        E: ContextExtractor + Clone + Send + Sync + 'static,
    {
        self.layer(axum::middleware::from_fn(move |req, next| {
            let middleware = middleware.clone();
            async move { middleware.handle(req, next).await }
        }))
    }
}
