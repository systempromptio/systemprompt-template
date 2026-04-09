use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::types::{PaddleCustomerRow, SubscriptionRow};

pub struct UpsertSubscriptionParams<'a> {
    pub user_id: &'a UserId,
    pub paddle_subscription_id: &'a str,
    pub paddle_customer_id: &'a str,
    pub plan_id: Option<uuid::Uuid>,
    pub status: &'a str,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub paddle_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpsertPlanParams<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub description: Option<&'a str>,
    pub paddle_product_id: &'a str,
    pub paddle_price_id: &'a str,
    pub amount_cents: i32,
    pub currency: &'a str,
    pub billing_interval: &'a str,
    pub features: &'a serde_json::Value,
    pub limits: &'a serde_json::Value,
    pub sort_order: i32,
}

pub async fn get_or_create_paddle_customer(
    pool: &PgPool,
    user_id: &UserId,
    email: &str,
) -> Result<PaddleCustomerRow, sqlx::Error> {
    sqlx::query_as!(
        PaddleCustomerRow,
        r#"INSERT INTO marketplace.paddle_customers (user_id, email) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET updated_at = now() RETURNING id, user_id, paddle_customer_id, email, name"#,
        user_id as &UserId,
        email,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_paddle_customer_id(
    pool: &PgPool,
    user_id: &UserId,
    paddle_customer_id: &str,
) -> Result<(), sqlx::Error> {
    let existing = sqlx::query_scalar!(
        "SELECT user_id FROM marketplace.paddle_customers WHERE paddle_customer_id = $1 AND user_id != $2",
        paddle_customer_id,
        user_id as &UserId,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(other_user_id) = existing {
        tracing::warn!(
            %paddle_customer_id,
            current_user = %user_id,
            existing_user = %other_user_id,
            "Paddle customer ID already linked to another user — updating to current user"
        );
        sqlx::query!(
            "UPDATE marketplace.paddle_customers SET paddle_customer_id = NULL, updated_at = now() WHERE paddle_customer_id = $1 AND user_id != $2",
            paddle_customer_id,
            user_id as &UserId,
        )
        .execute(pool)
        .await?;
    }

    sqlx::query!(
        "UPDATE marketplace.paddle_customers SET paddle_customer_id = $2, updated_at = now() WHERE user_id = $1",
        user_id as &UserId,
        paddle_customer_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_subscription(
    pool: &PgPool,
    params: &UpsertSubscriptionParams<'_>,
) -> Result<SubscriptionRow, sqlx::Error> {
    sqlx::query_as!(
        SubscriptionRow,
        r#"INSERT INTO marketplace.subscriptions (user_id, paddle_subscription_id, paddle_customer_id, plan_id, status, current_period_start, current_period_end, paddle_data) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (paddle_subscription_id) DO UPDATE SET plan_id = COALESCE($4, marketplace.subscriptions.plan_id), status = $5, current_period_start = COALESCE($6, marketplace.subscriptions.current_period_start), current_period_end = COALESCE($7, marketplace.subscriptions.current_period_end), paddle_data = COALESCE($8, marketplace.subscriptions.paddle_data), updated_at = now() RETURNING id, user_id, paddle_subscription_id, paddle_customer_id, plan_id, status, current_period_start, current_period_end, cancel_at, paddle_data, created_at, updated_at"#,
        params.user_id as &UserId,
        params.paddle_subscription_id,
        params.paddle_customer_id,
        params.plan_id,
        params.status,
        params.period_start,
        params.period_end,
        params.paddle_data.clone() as Option<serde_json::Value>,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_subscription_status(
    pool: &PgPool,
    paddle_subscription_id: &str,
    status: &str,
    cancel_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE marketplace.subscriptions SET status = $2, cancel_at = COALESCE($3, cancel_at), updated_at = now() WHERE paddle_subscription_id = $1",
        paddle_subscription_id,
        status,
        cancel_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn store_webhook_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    payload: &serde_json::Value,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO marketplace.paddle_webhook_events (event_id, event_type, payload) VALUES ($1, $2, $3) ON CONFLICT (event_id) DO NOTHING",
        event_id,
        event_type,
        payload,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_webhook_processed(pool: &PgPool, event_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE marketplace.paddle_webhook_events SET status = 'processed', processed_at = now() WHERE event_id = $1",
        event_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_webhook_failed(
    pool: &PgPool,
    event_id: &str,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE marketplace.paddle_webhook_events SET status = 'failed', error_message = $2, processed_at = now() WHERE event_id = $1",
        event_id,
        error_message,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_plan(pool: &PgPool, params: &UpsertPlanParams<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, currency, billing_interval, features, limits, sort_order, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true) ON CONFLICT (name) DO UPDATE SET display_name = $2, description = $3, paddle_product_id = $4, paddle_price_id = $5, amount_cents = $6, currency = $7, billing_interval = $8, features = $9, limits = $10, sort_order = $11, is_active = true, updated_at = now()",
        params.name,
        params.display_name,
        params.description,
        params.paddle_product_id,
        params.paddle_price_id,
        params.amount_cents,
        params.currency,
        params.billing_interval,
        params.features,
        params.limits,
        params.sort_order,
    )
    .execute(pool)
    .await?;
    Ok(())
}
