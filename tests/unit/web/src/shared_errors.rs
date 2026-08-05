//! The domain error enums hold a single transparent `Infra` variant instead of
//! their own copies of the infrastructure failures, so `code` and
//! `is_retryable` must delegate to `InfraError` rather than flattening every
//! infrastructure failure into one opaque internal error.

use systemprompt::traits::ExtensionError;
use systemprompt_web_shared::error::{BlogError, InfraError, MarketplaceError};

fn io_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, "missing file")
}

fn json_error() -> serde_json::Error {
    serde_json::from_str::<serde_json::Value>("{ not json").unwrap_err()
}

#[test]
fn infra_error_codes_name_the_failing_subsystem() {
    assert_eq!(InfraError::from(io_error()).code(), "IO_ERROR");
    assert_eq!(InfraError::from(json_error()).code(), "JSON_ERROR");
}

#[test]
fn only_transient_infrastructure_failures_are_retryable() {
    // IO can succeed on a retry; a malformed payload never will.
    assert!(InfraError::from(io_error()).is_retryable());
    assert!(!InfraError::from(json_error()).is_retryable());
}

#[test]
fn marketplace_error_codes_distinguish_conflict_from_bad_request() {
    assert_eq!(
        MarketplaceError::Internal("boom".to_owned()).code(),
        "INTERNAL_ERROR"
    );
    assert_eq!(
        MarketplaceError::BadRequest("bad".to_owned()).code(),
        "BAD_REQUEST"
    );
    assert_eq!(
        MarketplaceError::NotFound("gone".to_owned()).code(),
        "NOT_FOUND"
    );
    assert_eq!(
        MarketplaceError::Conflict("seats full".to_owned()).code(),
        "CONFLICT"
    );
    assert_eq!(
        MarketplaceError::Crypto("key".to_owned()).code(),
        "CRYPTO_ERROR"
    );
    // The infra code passes through rather than collapsing to INTERNAL_ERROR.
    assert_eq!(MarketplaceError::from(io_error()).code(), "IO_ERROR");
}

#[test]
fn marketplace_error_statuses_match_their_meaning() {
    assert_eq!(
        MarketplaceError::BadRequest("bad".to_owned())
            .status()
            .as_u16(),
        400
    );
    assert_eq!(
        MarketplaceError::NotFound("gone".to_owned())
            .status()
            .as_u16(),
        404
    );
    assert_eq!(
        MarketplaceError::Conflict("full".to_owned())
            .status()
            .as_u16(),
        409
    );
    assert_eq!(
        MarketplaceError::Crypto("key".to_owned()).status().as_u16(),
        500
    );
    assert_eq!(MarketplaceError::from(io_error()).status().as_u16(), 500);
}

#[test]
fn marketplace_error_is_retryable_only_through_a_retryable_infra_cause() {
    assert!(MarketplaceError::from(io_error()).is_retryable());
    assert!(!MarketplaceError::from(json_error()).is_retryable());
    assert!(!MarketplaceError::Internal("boom".to_owned()).is_retryable());
}

#[test]
fn blog_error_codes_and_statuses_split_not_found_from_invalid() {
    assert_eq!(
        BlogError::DatabaseNotPostgres.code(),
        "DATABASE_NOT_POSTGRES"
    );
    assert_eq!(BlogError::DatabaseNotPostgres.status().as_u16(), 500);
    assert_eq!(
        BlogError::ContentNotFound("slug".to_owned()).code(),
        "CONTENT_NOT_FOUND"
    );
    assert_eq!(
        BlogError::LinkNotFound("abc".to_owned()).status().as_u16(),
        404
    );
    assert_eq!(
        BlogError::InvalidRequest("nope".to_owned())
            .status()
            .as_u16(),
        400
    );
    assert_eq!(
        BlogError::Validation("nope".to_owned()).code(),
        "VALIDATION_ERROR"
    );
    assert_eq!(BlogError::Parse("nope".to_owned()).status().as_u16(), 400);
}

#[test]
fn blog_error_retryability_follows_its_infra_cause() {
    assert!(BlogError::from(io_error()).is_retryable());
    assert!(!BlogError::from(json_error()).is_retryable());
    assert!(!BlogError::DatabaseNotPostgres.is_retryable());
    assert_eq!(BlogError::from(json_error()).code(), "JSON_ERROR");
}
