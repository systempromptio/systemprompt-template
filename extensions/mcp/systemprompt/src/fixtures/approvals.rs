//! Approval-bound fixture writes and immutable replay receipts.

use systemprompt::database::DbPool;
use systemprompt::evaluation::repository::experiments::{
    ApprovalAuthorization, EvaluationLifecycleRepository,
};
use systemprompt::identifiers::{ActorKind, EvalApprovalId, EvalExecutionId};
use systemprompt::models::execution::context::RequestContext;

use super::digest;
use super::error::FixtureError;

#[derive(Debug)]
pub(super) struct AuthorizationGrant {
    id: EvalApprovalId,
    execution_id: EvalExecutionId,
    operation_digest: String,
}

pub(super) struct ApprovalOperation<'a> {
    pub fixture_key: &'a str,
    pub action: &'a str,
    // JSON: Approval digests bind the exact arbitrary fixture write document.
    pub value: &'a serde_json::Value,
    pub precondition: &'a str,
}

pub(super) async fn authorize_write(
    pool: &DbPool,
    context: &RequestContext,
    operation: ApprovalOperation<'_>,
) -> Result<AuthorizationGrant, FixtureError> {
    let ActorKind::Job { job_name } = &context.actor().kind else {
        return Err(FixtureError::Rejected(
            "Fixture writes require an evaluator execution",
        ));
    };
    let execution = job_name
        .strip_prefix("evaluation:")
        .ok_or_else(|| FixtureError::Rejected("Fixture writes require an evaluator execution"))?;
    let write = pool
        .write_pool()
        .ok_or_else(|| FixtureError::Unavailable("Writable evaluation database unavailable"))?;
    let lifecycle = EvaluationLifecycleRepository::new(write.as_ref().clone());
    let payload = serde_json::json!({"fixture_key":operation.fixture_key,"action":operation.action,"value":operation.value});
    let execution_id = EvalExecutionId::new(execution);
    let operation_digest = digest(&payload)?;
    match lifecycle
        .authorize_operation(
            context.user_id(),
            &execution_id,
            &payload,
            operation.precondition,
        )
        .await?
    {
        ApprovalAuthorization::Authorized(id) => Ok(AuthorizationGrant {
            id,
            execution_id,
            operation_digest,
        }),
        ApprovalAuthorization::Pending(id) => Err(FixtureError::ApprovalRequired(id)),
    }
}

// JSON: Stored receipts return the exact approved operation output.
pub(super) async fn approved_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: &str,
    grant: &AuthorizationGrant,
) -> Result<Option<serde_json::Value>, FixtureError> {
    let row = sqlx::query!(r#"SELECT a.status,r.operation_digest AS "operation_digest?",r.output AS "output?" FROM eval_execution_approvals a LEFT JOIN eval_approved_operation_receipts r ON r.approval_id=a.id WHERE a.id=$1 AND a.owner_id=$2 AND a.execution_id=$3 FOR UPDATE OF a"#,
        grant.id.as_str(), owner, grant.execution_id.as_str()).fetch_optional(&mut **tx).await?
        .ok_or_else(|| FixtureError::Rejected("Approved operation is unavailable"))?;
    let output = row.output;
    let receipt_digest = row.operation_digest;
    if let Some(output) = output {
        if receipt_digest.as_deref() != Some(&grant.operation_digest) {
            return Err(FixtureError::Rejected(
                "Approved operation receipt integrity failure",
            ));
        }
        return Ok(Some(output));
    }
    if row.status != "approved" {
        return Err(FixtureError::Rejected(
            "Approved write has no receipt; automatic replay is forbidden",
        ));
    }
    Ok(None)
}

pub(super) async fn commit_approved_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    grant: &AuthorizationGrant,
    // JSON: Stored receipts retain the exact approved operation output.
    output: &serde_json::Value,
) -> Result<(), FixtureError> {
    sqlx::query!("INSERT INTO eval_approved_operation_receipts(approval_id,execution_id,operation_digest,output) VALUES($1,$2,$3,$4)",
        grant.id.as_str(), grant.execution_id.as_str(), &grant.operation_digest, output).execute(&mut **tx).await?;
    let changed = sqlx::query!(
        "UPDATE eval_execution_approvals SET status='consumed' WHERE id=$1 AND status='approved'",
        grant.id.as_str()
    )
    .execute(&mut **tx)
    .await?;
    if changed.rows_affected() != 1 {
        return Err(FixtureError::Rejected(
            "Approval consumption raced with another writer",
        ));
    }
    Ok(())
}
