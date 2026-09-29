//! Publication lifecycle evidence.
//!
//! Every reviewed generation of every managed resource, with the delivery
//! state and receipt count that generation earned and the content hash the
//! reviewer recorded against it.
//!
//! `comparison_evidence` is the reviewer's verbatim JSON and its `recorded`
//! map is serde-flattened, so the provenance hashes sit at the top level of
//! the document rather than under a `recorded` key.

use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::ManagedResourceId;
use systemprompt::marketplace::inventory::InstallationCoverage;

#[derive(Debug, Clone)]
pub struct PublicationLifecycleRow {
    pub resource_id: String,
    pub resource_key: String,
    pub publication_id: String,
    pub review_id: String,
    pub generation: i64,
    pub action: String,
    pub revision_id: Option<String>,
    pub bundle_digest: Option<String>,
    pub reviewer_id: String,
    pub limitations: String,
    pub distribution_status: String,
    pub installation_receipts: i64,
    pub provenance_hash: Option<String>,
}

pub async fn list_publication_lifecycle(
    pool: &PgPool,
    owner: &str,
) -> Result<Vec<PublicationLifecycleRow>, sqlx::Error> {
    let rows = sqlx::query!(r#"SELECT m.id AS resource_id,m.resource_key,p.id AS publication_id,p.review_id,p.generation,p.action,p.revision_id,p.bundle_digest,r.reviewer_id,r.limitations,COALESCE(r.comparison_evidence->>'composed_hash',r.comparison_evidence->>'bundle_content_hash') AS provenance_hash,COALESCE(d.status,CASE WHEN o.delivered_at IS NOT NULL THEN 'distributed' ELSE 'approved' END) AS "distribution_status!",count(i.id)::BIGINT AS "installation_receipts!" FROM managed_publications p JOIN managed_resources m ON m.id=p.resource_id JOIN managed_publication_reviews r ON r.id=p.review_id LEFT JOIN managed_distribution_outbox o ON o.publication_id=p.id LEFT JOIN managed_distribution_deliveries d ON d.outbox_id=o.id LEFT JOIN managed_installation_receipts i ON i.publication_id=p.id WHERE p.owner_id=$1 GROUP BY m.id,m.resource_key,p.id,p.review_id,p.generation,p.action,p.revision_id,p.bundle_digest,r.reviewer_id,r.limitations,r.comparison_evidence,d.status,o.delivered_at ORDER BY m.resource_key ASC,p.generation DESC,p.created_at DESC"#,
        owner).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| PublicationLifecycleRow {
            resource_id: row.resource_id,
            resource_key: row.resource_key,
            publication_id: row.publication_id,
            review_id: row.review_id,
            generation: row.generation,
            action: row.action,
            revision_id: row.revision_id,
            bundle_digest: row.bundle_digest,
            reviewer_id: row.reviewer_id,
            limitations: row.limitations,
            distribution_status: row.distribution_status,
            installation_receipts: row.installation_receipts,
            provenance_hash: row.provenance_hash,
        })
        .collect())
}

/// Device coverage for one managed resource: how many entitled devices
/// acknowledged the currently published generation.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceCoverageRow {
    pub resource_id: ManagedResourceId,
    pub resource_key: String,
    pub body: Json<InstallationCoverage>,
}

pub async fn list_resource_coverage(
    pool: &PgPool,
    owner: &str,
) -> Result<Vec<ResourceCoverageRow>, sqlx::Error> {
    sqlx::query_as!(
        ResourceCoverageRow,
        r#"SELECT c.resource_id AS "resource_id: ManagedResourceId", m.resource_key,
                  c.body AS "body: Json<InstallationCoverage>"
           FROM managed_installation_coverage c
           JOIN managed_resources m ON m.owner_id = c.owner_id AND m.id = c.resource_id
           WHERE c.owner_id = $1 ORDER BY m.resource_key"#,
        owner,
    )
    .fetch_all(pool)
    .await
}
