use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct JwtMetrics {
    inner: Arc<JwtMetricsInner>,
}

#[derive(Debug)]
struct JwtMetricsInner {
    header_extraction_total: AtomicU64,
    cookie_extraction_total: AtomicU64,
    mcp_proxy_extraction_total: AtomicU64,
    extraction_failures_total: AtomicU64,
    jwt_validation_successes_total: AtomicU64,
    jwt_validation_failures_total: AtomicU64,
    invalid_signature_total: AtomicU64,
    expired_token_total: AtomicU64,
    missing_claims_total: AtomicU64,
    session_reuse_total: AtomicU64,
    new_session_total: AtomicU64,
    anonymous_session_total: AtomicU64,
}

impl Default for JwtMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl JwtMetrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(JwtMetricsInner {
                header_extraction_total: AtomicU64::new(0),
                cookie_extraction_total: AtomicU64::new(0),
                mcp_proxy_extraction_total: AtomicU64::new(0),
                extraction_failures_total: AtomicU64::new(0),
                jwt_validation_successes_total: AtomicU64::new(0),
                jwt_validation_failures_total: AtomicU64::new(0),
                invalid_signature_total: AtomicU64::new(0),
                expired_token_total: AtomicU64::new(0),
                missing_claims_total: AtomicU64::new(0),
                session_reuse_total: AtomicU64::new(0),
                new_session_total: AtomicU64::new(0),
                anonymous_session_total: AtomicU64::new(0),
            }),
        }
    }

    pub fn increment_header_extraction(&self) {
        self.inner
            .header_extraction_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_cookie_extraction(&self) {
        self.inner
            .cookie_extraction_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_mcp_proxy_extraction(&self) {
        self.inner
            .mcp_proxy_extraction_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_extraction_failure(&self) {
        self.inner
            .extraction_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_jwt_validation_success(&self) {
        self.inner
            .jwt_validation_successes_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_jwt_validation_failure(&self) {
        self.inner
            .jwt_validation_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_invalid_signature(&self) {
        self.inner
            .invalid_signature_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_expired_token(&self) {
        self.inner
            .expired_token_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_missing_claims(&self) {
        self.inner
            .missing_claims_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_session_reuse(&self) {
        self.inner
            .session_reuse_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_new_session(&self) {
        self.inner.new_session_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_anonymous_session(&self) {
        self.inner
            .anonymous_session_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_header_extraction_total(&self) -> u64 {
        self.inner.header_extraction_total.load(Ordering::Relaxed)
    }

    pub fn get_cookie_extraction_total(&self) -> u64 {
        self.inner.cookie_extraction_total.load(Ordering::Relaxed)
    }

    pub fn get_mcp_proxy_extraction_total(&self) -> u64 {
        self.inner
            .mcp_proxy_extraction_total
            .load(Ordering::Relaxed)
    }

    pub fn get_extraction_failures_total(&self) -> u64 {
        self.inner.extraction_failures_total.load(Ordering::Relaxed)
    }

    pub fn get_jwt_validation_successes_total(&self) -> u64 {
        self.inner
            .jwt_validation_successes_total
            .load(Ordering::Relaxed)
    }

    pub fn get_jwt_validation_failures_total(&self) -> u64 {
        self.inner
            .jwt_validation_failures_total
            .load(Ordering::Relaxed)
    }

    pub fn get_invalid_signature_total(&self) -> u64 {
        self.inner.invalid_signature_total.load(Ordering::Relaxed)
    }

    pub fn get_expired_token_total(&self) -> u64 {
        self.inner.expired_token_total.load(Ordering::Relaxed)
    }

    pub fn get_missing_claims_total(&self) -> u64 {
        self.inner.missing_claims_total.load(Ordering::Relaxed)
    }

    pub fn get_session_reuse_total(&self) -> u64 {
        self.inner.session_reuse_total.load(Ordering::Relaxed)
    }

    pub fn get_new_session_total(&self) -> u64 {
        self.inner.new_session_total.load(Ordering::Relaxed)
    }

    pub fn get_anonymous_session_total(&self) -> u64 {
        self.inner.anonymous_session_total.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> JwtMetricsSnapshot {
        JwtMetricsSnapshot {
            header_extraction_total: self.get_header_extraction_total(),
            cookie_extraction_total: self.get_cookie_extraction_total(),
            mcp_proxy_extraction_total: self.get_mcp_proxy_extraction_total(),
            extraction_failures_total: self.get_extraction_failures_total(),
            jwt_validation_successes_total: self.get_jwt_validation_successes_total(),
            jwt_validation_failures_total: self.get_jwt_validation_failures_total(),
            invalid_signature_total: self.get_invalid_signature_total(),
            expired_token_total: self.get_expired_token_total(),
            missing_claims_total: self.get_missing_claims_total(),
            session_reuse_total: self.get_session_reuse_total(),
            new_session_total: self.get_new_session_total(),
            anonymous_session_total: self.get_anonymous_session_total(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JwtMetricsSnapshot {
    pub header_extraction_total: u64,
    pub cookie_extraction_total: u64,
    pub mcp_proxy_extraction_total: u64,
    pub extraction_failures_total: u64,
    pub jwt_validation_successes_total: u64,
    pub jwt_validation_failures_total: u64,
    pub invalid_signature_total: u64,
    pub expired_token_total: u64,
    pub missing_claims_total: u64,
    pub session_reuse_total: u64,
    pub new_session_total: u64,
    pub anonymous_session_total: u64,
}

impl JwtMetricsSnapshot {
    pub fn total_extraction_attempts(&self) -> u64 {
        self.header_extraction_total
            + self.cookie_extraction_total
            + self.mcp_proxy_extraction_total
            + self.extraction_failures_total
    }

    pub fn successful_extractions(&self) -> u64 {
        self.header_extraction_total
            + self.cookie_extraction_total
            + self.mcp_proxy_extraction_total
    }

    pub fn extraction_success_rate(&self) -> f64 {
        let total = self.total_extraction_attempts();
        if total == 0 {
            0.0
        } else {
            (self.successful_extractions() as f64 / total as f64) * 100.0
        }
    }

    pub fn jwt_validation_success_rate(&self) -> f64 {
        let total = self.jwt_validation_successes_total + self.jwt_validation_failures_total;
        if total == 0 {
            0.0
        } else {
            (self.jwt_validation_successes_total as f64 / total as f64) * 100.0
        }
    }

    pub fn session_creation_breakdown(&self) -> SessionCreationBreakdown {
        SessionCreationBreakdown {
            new_sessions: self.new_session_total,
            reused_sessions: self.session_reuse_total,
            anonymous_sessions: self.anonymous_session_total,
            total_sessions: self.new_session_total
                + self.session_reuse_total
                + self.anonymous_session_total,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionCreationBreakdown {
    pub new_sessions: u64,
    pub reused_sessions: u64,
    pub anonymous_sessions: u64,
    pub total_sessions: u64,
}

impl SessionCreationBreakdown {
    pub fn reuse_percentage(&self) -> f64 {
        if self.total_sessions == 0 {
            0.0
        } else {
            (self.reused_sessions as f64 / self.total_sessions as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = JwtMetrics::new();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.header_extraction_total, 0);
        assert_eq!(snapshot.cookie_extraction_total, 0);
    }

    #[test]
    fn test_metrics_increment() {
        let metrics = JwtMetrics::new();
        metrics.increment_header_extraction();
        metrics.increment_header_extraction();
        metrics.increment_cookie_extraction();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.header_extraction_total, 2);
        assert_eq!(snapshot.cookie_extraction_total, 1);
    }

    #[test]
    fn test_extraction_success_rate() {
        let metrics = JwtMetrics::new();
        metrics.increment_header_extraction();
        metrics.increment_header_extraction();
        metrics.increment_extraction_failure();

        let snapshot = metrics.snapshot();
        let rate = snapshot.extraction_success_rate();
        assert!((rate - 66.66666666666666).abs() < 0.01);
    }

    #[test]
    fn test_session_creation_breakdown() {
        let metrics = JwtMetrics::new();
        metrics.increment_new_session();
        metrics.increment_new_session();
        metrics.increment_session_reuse();
        metrics.increment_session_reuse();
        metrics.increment_session_reuse();
        metrics.increment_anonymous_session();

        let snapshot = metrics.snapshot();
        let breakdown = snapshot.session_creation_breakdown();
        assert_eq!(breakdown.new_sessions, 2);
        assert_eq!(breakdown.reused_sessions, 3);
        assert_eq!(breakdown.anonymous_sessions, 1);
        assert_eq!(breakdown.total_sessions, 6);
        let reuse_pct = breakdown.reuse_percentage();
        assert!((reuse_pct - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_clone() {
        let metrics = JwtMetrics::new();
        metrics.increment_header_extraction();
        let metrics_clone = metrics.clone();
        assert_eq!(metrics_clone.get_header_extraction_total(), 1);
    }
}
