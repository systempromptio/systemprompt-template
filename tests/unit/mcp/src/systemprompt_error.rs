//! `SystempromptToolError` is what the CLI tool's failures normalise on, and
//! its three `ExtensionError` methods each drive a different consumer: `code`
//! is the stable string clients match on, `status` is the HTTP code the MCP
//! transport returns, and `is_retryable` decides whether the caller may try
//! again. The distinction that matters is that a failed CLI command is the
//! caller's fault (400, not retryable) while an IO fault is transient.

use axum::http::StatusCode;
use std::io;
use systemprompt::traits::ExtensionError;
use systemprompt_mcp_agent::error::SystempromptToolError;

fn variants() -> Vec<SystempromptToolError> {
    vec![
        SystempromptToolError::CommandFailed("exit 1".to_owned()),
        SystempromptToolError::NotFound("skill".to_owned()),
        SystempromptToolError::Io(io::Error::other("pipe closed")),
        SystempromptToolError::Serialization(
            serde_json::from_str::<serde_json::Value>("{").expect_err("invalid json"),
        ),
        SystempromptToolError::Internal("pool exhausted".to_owned()),
    ]
}

#[test]
fn every_variant_has_a_distinct_screaming_snake_code() {
    let codes: Vec<&str> = variants().iter().map(ExtensionError::code).collect();

    assert_eq!(
        codes,
        vec![
            "COMMAND_FAILED",
            "NOT_FOUND",
            "IO_ERROR",
            "SERIALIZATION_ERROR",
            "INTERNAL_ERROR",
        ]
    );
    for code in codes {
        assert_eq!(code, code.to_uppercase());
    }
}

#[test]
fn a_bad_command_is_the_callers_fault_and_everything_else_is_ours() {
    let statuses: Vec<StatusCode> = variants().iter().map(ExtensionError::status).collect();

    assert_eq!(
        statuses,
        vec![
            StatusCode::BAD_REQUEST,
            StatusCode::NOT_FOUND,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::INTERNAL_SERVER_ERROR,
        ]
    );
}

#[test]
fn only_io_faults_are_worth_retrying() {
    let retryable: Vec<bool> = variants()
        .iter()
        .map(ExtensionError::is_retryable)
        .collect();

    assert_eq!(retryable, vec![false, false, true, false, false]);

    for error in variants() {
        assert!(
            !error.to_string().is_empty(),
            "every variant must render a message the caller can log"
        );
    }
}
