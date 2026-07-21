//! Share-token versioning stored on `user_profile_ext`.
//!
//! Rotating `share_token_version` revokes every previously-issued share token
//! for that user; the public manifest endpoint rechecks the stored version
//! against the value encoded in the token.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// A user with no `user_profile_ext` row resolves to `Ok(None)` — absence of
/// a profile is not an error here.
pub async fn get_share_token_version(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<i32>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT share_token_version FROM user_profile_ext WHERE user_id = $1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.share_token_version))
}
