//! The breakdowns behind one organization: its departments, the entitlements
//! its plan projected, and where its inference spend went.
//!
//! Split from [`super::metrics`], which answers "how is every customer doing";
//! these answer "what is this one made of", and are read only by the detail
//! page.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::organization::organization_rule_type;

#[derive(Debug, Clone)]
pub struct OrganizationDepartment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: i64,
}

pub async fn list_organization_departments(
    pool: &PgPool,
    org_id: &str,
) -> Result<Vec<OrganizationDepartment>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            d.id AS "id!",
            d.name AS "name!",
            d.description AS "description!",
            (SELECT COUNT(*) FROM organization_members m
               JOIN user_profile_ext e ON e.user_id = m.user_id
              WHERE m.org_id = d.org_id AND e.department = d.name) AS "member_count!"
        FROM departments d
        WHERE d.org_id = $1
        ORDER BY d.name
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationDepartment {
            id: r.id,
            name: r.name,
            description: r.description,
            member_count: r.member_count,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct OrganizationEntitlement {
    pub entity_type: String,
    pub entity_id: String,
    pub access: String,
}

// Why: Read-only by design. Entitlement is authored in `plans.yaml` and derived
// from the plan the customer is on; a screen that let an operator edit the
// projection would be editing a value the next bootstrap pass overwrites.
pub async fn list_organization_entitlements(
    pool: &PgPool,
    slug: &str,
) -> Result<Vec<OrganizationEntitlement>, MarketplaceError> {
    let rule_type = organization_rule_type();
    let rows = sqlx::query!(
        r#"
        SELECT
            entity_type AS "entity_type!",
            entity_id AS "entity_id!",
            access AS "access!"
        FROM access_control_rules
        WHERE rule_type = $1 AND rule_value = $2
        ORDER BY entity_type, entity_id
        "#,
        rule_type.as_str(),
        slug,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationEntitlement {
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            access: r.access,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct OrganizationModelUsage {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

pub async fn list_organization_model_usage(
    pool: &PgPool,
    org_id: &str,
) -> Result<Vec<OrganizationModelUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.model AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        JOIN organization_members m ON m.user_id = r.user_id
        WHERE m.org_id = $1
          AND r.created_at >= NOW() - INTERVAL '30 days'
        GROUP BY r.model
        ORDER BY 4 DESC
        LIMIT 10
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationModelUsage {
            model: r.model,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}
