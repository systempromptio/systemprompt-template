//! Seat accounting and the seat limit.
//!
//! A seat is an *active* user with a membership row. Suspended and deleted
//! users do not consume one, so offboarding frees a seat without deleting the
//! audit trail that user's requests are attached to.
//!
//! The limit is enforced at every point that mints a seat, not at the point
//! that displays one. There are two such points — an operator or customer
//! admin creating a user, and SSO just-in-time provisioning — and a limit
//! checked at only one of them is not a limit, because the other is exactly
//! the path an enterprise customer's users arrive through.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy)]
pub struct SeatUsage {
    pub used: i64,
    pub limit: Option<i32>,
}

impl SeatUsage {
    #[must_use]
    pub const fn is_full(&self) -> bool {
        match self.limit {
            Some(limit) => self.used >= limit as i64,
            None => false,
        }
    }
}

pub async fn count_active_seats(pool: &PgPool, org_id: &str) -> Result<i64, MarketplaceError> {
    let used = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM organization_members m
        JOIN users u ON u.id = m.user_id
        WHERE m.org_id = $1 AND u.status = 'active'
        "#,
        org_id
    )
    .fetch_one(pool)
    .await?;
    Ok(used)
}

pub async fn get_seat_usage(pool: &PgPool, org_id: &str) -> Result<SeatUsage, MarketplaceError> {
    let limit = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(o.seat_limit_override, p.seat_limit) AS "seat_limit?"
        FROM organizations o
        LEFT JOIN plans p ON p.id = o.plan_id
        WHERE o.id = $1
        "#,
        org_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| MarketplaceError::NotFound(format!("organization {org_id}")))?;

    Ok(SeatUsage {
        used: count_active_seats(pool, org_id).await?,
        limit,
    })
}

// Why: A full plan is a conflict, not a bad request: the caller has nothing to
// correct, the customer needs to buy seats or deactivate someone, and the
// message says so.
//
// There is a race here between the check and the insert that would let two
// simultaneous invitations both land on the last seat. It is left open
// deliberately rather than papered over with a lock: overshooting a seat cap
// by one on a genuine race is a billing reconciliation, whereas serialising
// every user creation on a per-org lock is a cost paid on every request. The
// admin surface shows `used`/`limit`, so the overshoot is visible.
pub async fn assert_seat_available(pool: &PgPool, org_id: &str) -> Result<(), MarketplaceError> {
    let usage = get_seat_usage(pool, org_id).await?;
    if usage.is_full() {
        let limit = usage.limit.unwrap_or_default();
        return Err(MarketplaceError::Conflict(format!(
            "seat limit reached: {}/{limit} seats in use. Deactivate a user or raise the plan's \
             seat limit.",
            usage.used
        )));
    }
    Ok(())
}
