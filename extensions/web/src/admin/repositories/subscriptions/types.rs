use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlanRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub paddle_product_id: String,
    pub paddle_price_id: String,
    pub amount_cents: i32,
    pub currency: String,
    pub billing_interval: String,
    pub features: serde_json::Value, // JSON: DB jsonb column
    pub limits: serde_json::Value,   // JSON: DB jsonb column
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub paddle_subscription_id: Option<String>,
    pub paddle_customer_id: Option<String>,
    pub plan_id: Option<uuid::Uuid>,
    pub status: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at: Option<DateTime<Utc>>,
    pub paddle_data: Option<serde_json::Value>, // JSON: DB jsonb column (external API data)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaddleCustomerRow {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub paddle_customer_id: Option<String>,
    pub email: String,
    pub name: Option<String>,
}
