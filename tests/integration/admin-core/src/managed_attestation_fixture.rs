//! Retained application attestation fixture for publication repository tests.
//! This is not source-verification coverage: that boundary must reject forged
//! provenance independently of these repository-level publication tests.

use systemprompt::identifiers::{ResourceRevisionId, UserId};
use systemprompt::marketplace::managed::ManagedRepository;

pub(crate) async fn retain(
    pool: &sqlx::PgPool,
    repo: &ManagedRepository,
    owner: &UserId,
    revision: &ResourceRevisionId,
    experiment: &str,
) {
    let resource = repo.revision_resource(owner, revision).await.unwrap();
    let bundle = repo
        .get_revision_bundle(owner, revision)
        .await
        .unwrap()
        .digest()
        .unwrap();
    sqlx::query("INSERT INTO managed_evaluation_attestations(owner_id,resource_id,revision_id,bundle_digest,experiment_id,campaign_id,evidence_digest,source_commit,attested_by) VALUES($1,$2,$3,$4,$5,'fixture-campaign','fixture-evidence','fixture-commit',$1)")
        .bind(owner.as_str()).bind(resource.as_str()).bind(revision.as_str()).bind(bundle.as_str()).bind(experiment).execute(pool).await.unwrap();
}
