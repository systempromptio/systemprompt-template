pub mod ai_usage;
pub mod concurrency;
pub mod endpoints;
pub mod events;
pub mod geoip;
pub mod integrity;
/// Analytics integration tests module
///
/// Tests for:
/// - Session creation and deduplication
/// - Session tracking (request counts, timestamps, duration)
/// - Event recording (page views, errors)
/// - Endpoint tracking with response times
/// - UTM parameters and referrer tracking
/// - GeoIP enrichment
/// - Referential integrity
/// - Traffic tracking (production bug verification)
pub mod session_creation;
pub mod session_creation_failure;
pub mod session_deduplication;
pub mod traffic_tracking;
pub mod utm_attribution;
