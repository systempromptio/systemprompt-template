use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::types::{PaddleCustomerRow, PlanRow, SubscriptionRow};

pub async fn list_active_plans(pool: &PgPool) -> Result<Vec<PlanRow>, sqlx::Error> {
    sqlx::query_as!(
        PlanRow,
        "SELECT id, name, display_name, description, paddle_product_id, paddle_price_id,
                amount_cents, currency, billing_interval, features, limits, sort_order, is_active
         FROM marketplace.plans
         WHERE is_active = true
         ORDER BY sort_order ASC, amount_cents ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn find_plan_by_paddle_price_id(
    pool: &PgPool,
    price_id: &str,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as!(
        PlanRow,
        "SELECT id, name, display_name, description, paddle_product_id, paddle_price_id,
                amount_cents, currency, billing_interval, features, limits, sort_order, is_active
         FROM marketplace.plans
         WHERE paddle_price_id = $1",
        price_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_subscription_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<SubscriptionRow>, sqlx::Error> {
    sqlx::query_as!(
        SubscriptionRow,
        "SELECT id, user_id, paddle_subscription_id, paddle_customer_id, plan_id, status,
                current_period_start, current_period_end, cancel_at, paddle_data,
                created_at, updated_at
         FROM marketplace.subscriptions
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_subscription_by_paddle_id(
    pool: &PgPool,
    paddle_subscription_id: &str,
) -> Result<Option<SubscriptionRow>, sqlx::Error> {
    sqlx::query_as!(
        SubscriptionRow,
        "SELECT id, user_id, paddle_subscription_id, paddle_customer_id, plan_id, status,
                current_period_start, current_period_end, cancel_at, paddle_data,
                created_at, updated_at
         FROM marketplace.subscriptions
         WHERE paddle_subscription_id = $1",
        paddle_subscription_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_customer_by_paddle_id(
    pool: &PgPool,
    paddle_customer_id: &str,
) -> Result<Option<PaddleCustomerRow>, sqlx::Error> {
    sqlx::query_as!(
        PaddleCustomerRow,
        "SELECT id, user_id, paddle_customer_id, email, name
         FROM marketplace.paddle_customers
         WHERE paddle_customer_id = $1",
        paddle_customer_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_user_tier(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<crate::admin::types::UserTier>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT COALESCE(p.display_name, 'Premium') AS \"display_name!\", s.status
         FROM marketplace.subscriptions s
         LEFT JOIN marketplace.plans p ON p.id = s.plan_id
         WHERE s.user_id = $1 AND s.status = 'active'
         ORDER BY s.created_at DESC
         LIMIT 1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| crate::admin::types::UserTier {
        plan_name: r.display_name,
        status: r.status,
    }))
}
