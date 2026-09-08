//! The two halves of a user's role set, and the recomputation that joins
//! them.
//!
//! `users.roles` is the effective set every authorisation check reads, but it
//! has two independent writers: the directory projects roles from the AD
//! groups an assertion carried, and an admin grants roles by hand in the
//! dashboard. Only the manual half is stored — `user_manual_roles` — so the
//! directory half is whatever the effective set holds beyond it. That keeps
//! one row per manual grant and no second copy of what AD already says.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::constants::ROLE_PLATFORM_ADMIN;

pub async fn list_manual_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT role FROM user_manual_roles WHERE user_id = $1 ORDER BY role",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

// Why: The roles the directory holds for this user: the effective set minus
// what was granted by hand.
//
// Why derived rather than stored: the directory rewrites `users.roles` at
// every sign-in from the assertion alone, so a stored copy would be a second
// truth that can only go stale. A role held both ways counts as manual here,
// which is the safe direction — revoking it is refused by no rule, and the
// next sign-in restores it.
pub async fn list_directory_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    directory_roles_on(pool, user_id).await
}

async fn directory_roles_on<'e, E>(
    executor: E,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    // Why: only explicit directory memberships establish a directory-owned
    // grant. Existing local users' free-text roles remain editable.
    sqlx::query_scalar::<_, String>(
        r#"SELECT r FROM users u, UNNEST(u.roles) AS r
        WHERE u.id = $1
          AND (EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = u.id AND gm.source = 'adfs')
            OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.user_id = u.id AND pm.source = 'adfs'))
          AND NOT EXISTS (SELECT 1 FROM user_manual_roles m WHERE m.user_id = u.id AND m.role = r)
        ORDER BY r"#,
    )
    .bind(user_id.as_str())
    .fetch_all(executor)
    .await
}

pub async fn count_platform_admins(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!" FROM users WHERE $1 = ANY(roles)"#,
        ROLE_PLATFORM_ADMIN
    )
    .fetch_one(pool)
    .await
}

// Why: Replace this user's manual grants with `roles`.
//
// Why replace: the role editor sends the whole set it wants, so a diff here
// would only be a second place for the two to disagree.
//
// Why the directory half is read first: it is derived as "effective minus
// manual", so once the manual rows change the derivation would count a
// revoked manual grant as directory-held and keep it. Snapshotting it before
// the rewrite, inside the same transaction, is what makes a revoke stick.
pub async fn set_manual_roles(
    pool: &PgPool,
    user_id: &UserId,
    roles: &[String],
    granted_by: &UserId,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let directory = directory_roles_on(&mut *tx, user_id).await?;
    sqlx::query!(
        "DELETE FROM user_manual_roles WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO user_manual_roles (user_id, role, granted_by)
         SELECT $1, r, $3 FROM UNNEST($2::TEXT[]) AS r ON CONFLICT DO NOTHING",
        user_id.as_str(),
        roles,
        granted_by.as_str()
    )
    .execute(&mut *tx)
    .await?;
    write_effective_roles(&mut *tx, user_id, &directory).await?;
    tx.commit().await
}

// Why: Rewrite `users.roles` as the union of what the directory grants and what
// was granted by hand, and return the result.
//
// `directory_roles` is `Some` at sign-in, where the assertion is the whole
// truth about the directory half; `None` everywhere else, where the current
// effective set already carries it and only the manual half has moved.
pub async fn recompute_roles(
    pool: &PgPool,
    user_id: &UserId,
    directory_roles: Option<&[String]>,
) -> Result<Vec<String>, sqlx::Error> {
    let directory = match directory_roles {
        Some(roles) => roles.to_vec(),
        None => list_directory_roles(pool, user_id).await?,
    };
    write_effective_roles(pool, user_id, &directory).await
}

async fn write_effective_roles<'e, E>(
    executor: E,
    user_id: &UserId,
    directory: &[String],
) -> Result<Vec<String>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_scalar!(
        r#"
        UPDATE users SET roles = (
            SELECT COALESCE(ARRAY_AGG(DISTINCT r ORDER BY r), ARRAY[]::TEXT[])
            FROM (
                SELECT UNNEST($2::TEXT[]) AS r
                UNION
                SELECT role FROM user_manual_roles WHERE user_id = $1
            ) all_roles
        )
        WHERE id = $1
        RETURNING roles AS "roles!"
        "#,
        user_id.as_str(),
        directory
    )
    .fetch_one(executor)
    .await
}
