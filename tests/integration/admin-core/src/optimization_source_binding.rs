//! Content equivalence is independent of history; a claimed SHA is not proof.

use crate::fixtures::{insert_user, unclaimed_email, unique};
use systemprompt::identifiers::{EvalCampaignId, EvalExperimentId};
use systemprompt::marketplace::managed::evaluation::EvaluationAttestation;
use systemprompt::marketplace::managed::{AssetDigest, ManagedRepository};

#[tokio::test]
async fn source_attestation_rejects_unverified_commits_and_preserves_content_identity() {
    let db = template_test_common::db_or_skip!();
    let owner = insert_user(
        &db.pool,
        &unique("source-owner"),
        &unclaimed_email("source-owner"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let input =
        crate::managed_revisions::revision_input(&repo, &owner, &unique("source-skill")).await;
    let baseline = repo.create_revision(&owner, &input).await.unwrap();
    let mut equivalent_input = input.clone();
    equivalent_input.parent_id = Some(baseline.clone());
    equivalent_input.rationale = "Equivalent source history".to_owned();
    let candidate = repo
        .create_revision(&owner, &equivalent_input)
        .await
        .unwrap();
    let original = repo.get_revision_bundle(&owner, &baseline).await.unwrap();
    let equivalent = repo.get_revision_bundle(&owner, &candidate).await.unwrap();
    assert_ne!(original.digest().unwrap(), equivalent.digest().unwrap());
    assert_eq!(
        original.content_digest().unwrap(),
        equivalent.content_digest().unwrap()
    );
    let attestation = EvaluationAttestation {
        resource_id: input.resource_id,
        revision_id: candidate,
        bundle_digest: equivalent.digest().unwrap(),
        experiment_id: EvalExperimentId::generate(),
        campaign_id: EvalCampaignId::generate(),
        evidence_digest: AssetDigest::of(b"untrusted claim"),
        source_commit: "a".repeat(40),
    };
    assert!(
        repo.attest_evaluation(&owner, &owner, &attestation)
            .await
            .is_err()
    );
    db.cleanup().await;
}
