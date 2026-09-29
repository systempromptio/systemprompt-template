//! The Distribution tab joins recorded marketplace versions to the managed
//! publication lifecycle. It must show a real delivered generation while
//! excluding another resource the marketplace does not carry.

use std::collections::BTreeMap;

use axum::http::StatusCode;
use serde_json::json;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, ComparisonEvidence, ManagedRepository, NewResource, NewRevision,
    PublicationAction, PublicationRequest, ResourceKind, RevisionFiles, SnapshotProvenance,
    SourceSpec,
};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

async fn publish_skill(repo: &ManagedRepository, owner: &UserId, key: &str) -> String {
    let source = repo
        .register_source(
            owner,
            &seed::unique("distribution-source"),
            &SourceSpec::Managed,
        )
        .await
        .expect("register managed source");
    let snapshot = repo
        .capture_snapshot(
            owner,
            &source,
            &SnapshotProvenance {
                source_kind: "managed".to_owned(),
                commit: None,
                tree_digest: AssetDigest::of(b"distribution fixture"),
                importer_version: "contract".to_owned(),
            },
        )
        .await
        .expect("capture managed snapshot");
    let resource_id = repo
        .bind_resource(
            owner,
            &NewResource {
                source_id: source,
                upstream_key: key.to_owned(),
                kind: ResourceKind::Skill,
                resource_key: key.to_owned(),
            },
        )
        .await
        .expect("bind managed skill");
    let revision = repo
        .create_revision(
            owner,
            &NewRevision {
                resource_id: resource_id.clone(),
                snapshot_id: snapshot,
                parent_id: None,
                files: RevisionFiles(BTreeMap::from([(
                    "SKILL.md".to_owned(),
                    AssetFile {
                        bytes: b"Read the managed distribution evidence.".to_vec(),
                        media_type: "text/markdown".to_owned(),
                        executable: false,
                    },
                )])),
                dependencies: BTreeMap::new(),
                rationale: "contract lifecycle fixture".to_owned(),
            },
        )
        .await
        .expect("write managed revision");
    let publication = repo
        .review_and_publish(
            owner,
            owner,
            &PublicationRequest {
                resource_id,
                revision_id: Some(revision),
                action: PublicationAction::InitialAdoption,
                expected_generation: 0,
                operation_key: seed::unique("distribution-publication"),
                comparison_evidence: ComparisonEvidence {
                    recorded: BTreeMap::from([(
                        "composed_hash".to_owned(),
                        serde_json::Value::from("contract-provenance"),
                    )]),
                },
                limitations: "Contract distribution limitation".to_owned(),
            },
        )
        .await
        .expect("publish managed skill");
    let claim = repo
        .claim_distribution(owner, &seed::unique("distribution-claim"))
        .await
        .expect("claim delivery")
        .expect("new publication has an outbox row");
    repo.complete_distribution(owner, &claim, true, None)
        .await
        .expect("mark delivery distributed");
    publication.publication_id.as_str().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn marketplace_distribution_shows_only_its_delivered_managed_skill() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let owner = credentials.admin_user_id.clone();
    let app = App::new(&db.pool, credentials);
    let repository = ManagedRepository::new(&db.db_pool()).expect("managed repository");
    let in_scope = seed::unique("distribution-in-scope");
    let out_of_scope = seed::unique("distribution-out-of-scope");
    let publication = publish_skill(&repository, &owner, &in_scope).await;
    publish_skill(&repository, &owner, &out_of_scope).await;

    let marketplace = seed::unique("distribution-marketplace");
    sqlx::query(
        "INSERT INTO marketplace_versions
             (marketplace_id, content_hash, source, manifest, plugin_count, skill_count)
         VALUES ($1, $2, 'contract fixture', $3::jsonb, 1, 1)",
    )
    .bind(&marketplace)
    .bind("d".repeat(64))
    .bind(
        json!({
            "marketplace_id": marketplace,
            "name": "Distribution contract marketplace",
            "version": "1.0.0",
            "plugins": [{
                "plugin_id": "distribution-contract-plugin",
                "digest": "distribution-plugin",
                "skills": [{"skill_id": in_scope, "skill_key": "scoped", "digest": "scoped"}]
            }],
            "files": 1
        })
        .to_string(),
    )
    .execute(&*db.pool)
    .await
    .expect("record marketplace version");

    let path = format!("/admin/analysis/versions/{marketplace}?tab=distribution");
    let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "distribution page: {body}");
    assert!(body.contains(&in_scope), "scoped resource is shown: {body}");
    assert!(
        body.contains(&format!("title=\"{publication}\"")),
        "the page retains the exact delivered publication id"
    );
    assert!(body.contains(">distributed<"));
    assert!(body.contains("Initial adoption"));
    assert!(body.contains("Contract distribution limitation"));
    assert!(
        !body.contains(&out_of_scope),
        "the marketplace manifest does not leak another managed resource: {body}"
    );
    db.cleanup().await;
}
