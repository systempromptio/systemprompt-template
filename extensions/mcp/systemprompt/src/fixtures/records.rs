//! The four fixture reads and the two guarded record mutations.
//!
//! A mutation is authorised against the execution's approval, short-circuits
//! to the stored receipt when it already ran, and otherwise locks the record,
//! re-checks the precondition digest the client observed, applies the change
//! and commits the receipt in the same transaction — so a retried call can
//! never apply twice and a stale client can never overwrite a newer value.

use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::execution::context::RequestContext;

use super::approvals::{
    ApprovalOperation, approved_receipt, authorize_write, commit_approved_receipt,
};
use super::error::FixtureError;
use super::{FixtureInput, digest};

// JSON: fixture payloads are case-specific evidence documents.
pub(super) struct Evidence {
    pub(super) payload: serde_json::Value,
    pub(super) digest: String,
    pub(super) label: String,
    pub(super) readback_verified: bool,
    pub(super) message: &'static str,
}

impl Evidence {
    fn fresh(payload: serde_json::Value, digest: String, label: &str, verified: bool) -> Self {
        Self {
            payload,
            digest,
            label: label.to_owned(),
            readback_verified: verified,
            message: "Deterministic fixture evidence returned",
        }
    }
}

pub(super) async fn read_payload(
    pool: &sqlx::PgPool,
    owner: &UserId,
    input: &FixtureInput,
) -> Result<Evidence, FixtureError> {
    if input.value.is_some() {
        return Err(FixtureError::Rejected("Read operations reject values"));
    }
    let row = sqlx::query!(
        "SELECT payload,digest,evidence_label FROM eval_fixture_payloads \
         WHERE owner_id=$1 AND fixture_key=$2 AND ($3::TEXT IS NULL OR digest=$3) \
         ORDER BY created_at DESC LIMIT 1",
        owner.as_str(),
        &input.fixture_key,
        input.expected_digest.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or(FixtureError::Rejected("Fixture unavailable"))?;
    Ok(Evidence::fresh(
        row.payload,
        row.digest,
        &row.evidence_label,
        false,
    ))
}

pub(super) async fn read_record(
    pool: &sqlx::PgPool,
    owner: &UserId,
    input: &FixtureInput,
) -> Result<Evidence, FixtureError> {
    let value = current_record(pool, owner, &input.fixture_key).await?;
    let value_digest = digest(&value)?;
    Ok(Evidence::fresh(
        value,
        value_digest,
        "fixture:platform_test_record",
        false,
    ))
}

pub(super) enum Mutation {
    // JSON: the replacement document the client supplied.
    Write(serde_json::Value),
    Restore,
}

impl Mutation {
    const fn action(&self) -> &'static str {
        match self {
            Self::Write(_) => "write",
            Self::Restore => "restore",
        }
    }

    const fn label(&self) -> &'static str {
        match self {
            Self::Write(_) => "fixture:platform_test_record_write",
            Self::Restore => "fixture:platform_test_record_restore",
        }
    }

    const fn requested_value(&self) -> &serde_json::Value {
        match self {
            Self::Write(value) => value,
            Self::Restore => &serde_json::Value::Null,
        }
    }
}

pub(super) async fn mutate_record(
    pool: &DbPool,
    context: &RequestContext,
    input: &FixtureInput,
    mutation: Mutation,
) -> Result<Evidence, FixtureError> {
    let owner = context.user_id();
    let expected = input
        .expected_digest
        .as_deref()
        .ok_or(FixtureError::Rejected(
            "Mutations require the observed precondition digest",
        ))?;
    let grant = authorize_write(
        pool,
        context,
        ApprovalOperation {
            fixture_key: &input.fixture_key,
            action: mutation.action(),
            value: mutation.requested_value(),
            precondition: expected,
        },
    )
    .await?;
    let mut tx = pool.begin().await?;
    if let Some(receipt) = approved_receipt(&mut tx, owner.as_str(), &grant).await? {
        tx.commit().await?;
        let receipt_digest = digest(&receipt)?;
        return Ok(Evidence {
            payload: receipt,
            digest: receipt_digest,
            label: mutation.label().to_owned(),
            readback_verified: true,
            message: "Previously verified idempotent fixture receipt returned",
        });
    }
    let locked = sqlx::query_scalar!(
        "SELECT value FROM eval_fixture_test_records WHERE owner_id=$1 AND record_key=$2 FOR UPDATE",
        owner.as_str(),
        &input.fixture_key
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(FixtureError::Rejected("Test record unavailable"))?;
    if digest(&locked)? != expected {
        return Err(FixtureError::Rejected("Test record precondition changed"));
    }
    let readback = apply(&mut tx, owner, &input.fixture_key, &mutation).await?;
    let verified = match &mutation {
        Mutation::Write(value) => &readback == value,
        Mutation::Restore => true,
    };
    let readback_digest = digest(&readback)?;
    commit_approved_receipt(&mut tx, &grant, &readback).await?;
    tx.commit().await?;
    Ok(Evidence::fresh(
        readback,
        readback_digest,
        mutation.label(),
        verified,
    ))
}

// JSON: the stored record is a case-specific evidence document.
// JSON: the readback is the stored evidence document after the mutation.
async fn apply(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: &UserId,
    key: &str,
    mutation: &Mutation,
) -> Result<serde_json::Value, FixtureError> {
    let readback = match mutation {
        Mutation::Write(value) => {
            sqlx::query_scalar!(
                "UPDATE eval_fixture_test_records SET value=$3,updated_at=NOW() \
                 WHERE owner_id=$1 AND record_key=$2 RETURNING value",
                owner.as_str(),
                key,
                value
            )
            .fetch_one(&mut **tx)
            .await?
        },
        Mutation::Restore => {
            sqlx::query_scalar!(
                "UPDATE eval_fixture_test_records SET value=original_value,updated_at=NOW() \
                 WHERE owner_id=$1 AND record_key=$2 RETURNING value",
                owner.as_str(),
                key
            )
            .fetch_one(&mut **tx)
            .await?
        },
    };
    Ok(readback)
}

// JSON: the stored record is a case-specific evidence document.
async fn current_record(
    pool: &sqlx::PgPool,
    owner: &UserId,
    key: &str,
) -> Result<serde_json::Value, FixtureError> {
    sqlx::query_scalar!(
        "SELECT value FROM eval_fixture_test_records WHERE owner_id=$1 AND record_key=$2",
        owner.as_str(),
        key
    )
    .fetch_optional(pool)
    .await?
    .ok_or(FixtureError::Rejected("Test record unavailable"))
}
