use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PlanRow {
    pub plan_name: String,
    pub limits: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct SubscriptionTierRow {
    pub plan_name: String,
    pub status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    pub limits: Option<serde_json::Value>,
}

pub async fn find_role_based_plan(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<PlanRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT p.display_name AS "plan_name!", p.limits
           FROM marketplace.plans p
           WHERE p.role_name = ANY(
               SELECT UNNEST(roles) FROM users WHERE id = $1
           )
           AND p.is_active = true
           ORDER BY p.sort_order DESC
           LIMIT 1"#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| PlanRow {
        plan_name: r.plan_name,
        limits: r.limits,
    }))
}

pub async fn find_free_plan(pool: &PgPool) -> Result<Option<PlanRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT p.display_name AS "plan_name!", p.limits
           FROM marketplace.plans p
           WHERE p.name = 'free' AND p.is_active = true
           LIMIT 1"#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| PlanRow {
        plan_name: r.plan_name,
        limits: r.limits,
    }))
}

pub async fn find_subscription_tier(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<SubscriptionTierRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
             COALESCE(p.display_name, 'Free') AS "plan_name!",
             s.status,
             s.current_period_end,
             p.limits AS "limits?: serde_json::Value"
           FROM marketplace.subscriptions s
           LEFT JOIN marketplace.plans p ON p.id = s.plan_id
           WHERE s.user_id = $1
           ORDER BY s.created_at DESC
           LIMIT 1"#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| SubscriptionTierRow {
        plan_name: r.plan_name,
        status: r.status,
        current_period_end: r.current_period_end,
        limits: r.limits,
    }))
}
