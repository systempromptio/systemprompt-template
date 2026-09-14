//! Hands a freshly published bundle to the evaluator as a workspace, so the
//! revision the reviewer approved is exactly the one experiments run against.

use systemprompt::evaluation::repository::experiments::ManagedWorkspaceRegistration;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::managed::PublicationDecision;

use crate::error::{AdminError, AdminResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;

// Why: a decision that did not publish (a rejection, or a review awaiting a
// second reviewer) carries no revision or digest and registers nothing.
pub(crate) async fn register_published_workspace(
    managed: &ManagedState,
    evaluations: &EvaluationState,
    user_id: &UserId,
    decision: &PublicationDecision,
) -> AdminResult<()> {
    let (Some(revision), Some(digest)) = (&decision.revision_id, &decision.bundle_digest) else {
        return Ok(());
    };
    let bundle = managed
        .repository
        .get_publication_bundle(user_id, &decision.resource_id, decision.generation, digest)
        .await?;
    let file_count = bundle
        .revisions
        .values()
        .map(|manifest| manifest.files.len())
        .sum();
    let byte_count = bundle
        .revisions
        .values()
        .flat_map(|manifest| manifest.files.values())
        .try_fold(0usize, |total, file| {
            usize::try_from(file.bytes)
                .ok()
                .and_then(|bytes| total.checked_add(bytes))
        })
        .ok_or_else(|| AdminError::BadRequest("Published bundle size overflow".to_owned()))?;
    evaluations
        .evidence
        .register_managed_workspace(
            user_id,
            &ManagedWorkspaceRegistration {
                managed_revision_id: revision.as_str(),
                publication_generation: Some(decision.generation),
                manifest: &serde_json::to_value(&bundle).map_err(AdminError::internal)?,
                expected_digest: digest.as_str(),
                file_count,
                byte_count,
            },
        )
        .await?;
    Ok(())
}
