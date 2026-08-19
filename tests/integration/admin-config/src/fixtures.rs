//! Raw-SQL row builders.
//!
//! Every test owns its own database, but that database is not empty: the web
//! extension's migrations seed a default department and dashboard rows.
//! Fixtures therefore mint their own ids and assertions name those ids.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[must_use]
pub fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0)
        .single()
        .expect("fixed timestamp is unambiguous")
}

#[must_use]
pub fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}

#[must_use]
pub fn user_id(raw: &str) -> UserId {
    UserId::new(raw.to_owned())
}

pub async fn insert_user(pool: &PgPool, id: &str) {
    sqlx::query(
        "INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
         VALUES ($1, $2, $2, $3, 'active', TRUE, ARRAY['user'])",
    )
    .bind(id)
    .bind(format!("{id}@example.test"))
    .bind(format!("User {id}"))
    .execute(pool)
    .await
    .expect("insert user");
}

// One `ai_requests` row. Defaults describe a completed, costed request; each
// test overrides only the columns its assertion reads.
pub struct RequestSeed<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cache_read_tokens: i32,
    pub cost_microdollars: i64,
    pub status: &'a str,
    pub created_at: DateTime<Utc>,
}

impl<'a> RequestSeed<'a> {
    #[must_use]
    pub fn new(id: &'a str, user_id: &'a str, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            user_id,
            provider: Some("anthropic"),
            model: Some("claude-sonnet-4-5-20250929"),
            input_tokens: 100,
            output_tokens: 20,
            cache_read_tokens: 0,
            cost_microdollars: 1_000,
            status: "completed",
            created_at,
        }
    }
}

pub async fn insert_request(pool: &PgPool, seed: &RequestSeed<'_>) {
    sqlx::query(
        "INSERT INTO ai_requests
            (id, request_id, user_id, context_id, provider, model, input_tokens, output_tokens,
             tokens_used, cache_read_tokens, cost_microdollars, status,
             actor_kind, actor_id, created_at, updated_at)
         VALUES ($1, $1, $2, md5($2)::uuid, $3, $4, $5, $6, $7, $8, $9, $10, 'user', $2, $11, $11)",
    )
    .bind(seed.id)
    .bind(seed.user_id)
    .bind(seed.provider)
    .bind(seed.model)
    .bind(seed.input_tokens)
    .bind(seed.output_tokens)
    .bind(seed.input_tokens + seed.output_tokens)
    .bind(seed.cache_read_tokens)
    .bind(seed.cost_microdollars)
    .bind(seed.status)
    .bind(seed.created_at)
    .execute(pool)
    .await
    .expect("insert ai request");
}

// Registers the catalog row `access_control_rules` foreign-keys against.
pub async fn insert_acl_entity(pool: &PgPool, entity_type: &str, entity_id: &str, default: bool) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, $3, 'test')
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET default_included = EXCLUDED.default_included",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(default)
    .execute(pool)
    .await
    .expect("insert access-control entity");
}

struct EnvVar<'a> {
    plugin: &'a str,
    name: &'a str,
    value: &'a str,
    secret: bool,
}

pub async fn insert_env_var(
    pool: &PgPool,
    user: &str,
    plugin: &str,
    name: &str,
    value: &str,
) -> String {
    let var = EnvVar {
        plugin,
        name,
        value,
        secret: false,
    };
    insert_env_var_row(pool, user, &var).await
}

pub async fn insert_secret_env_var(
    pool: &PgPool,
    user: &str,
    plugin: &str,
    name: &str,
    value: &str,
) -> String {
    let var = EnvVar {
        plugin,
        name,
        value,
        secret: true,
    };
    insert_env_var_row(pool, user, &var).await
}

async fn insert_env_var_row(pool: &PgPool, user: &str, var: &EnvVar<'_>) -> String {
    let id = unique("env");
    sqlx::query(
        "INSERT INTO plugin_env_vars (id, user_id, plugin_id, var_name, var_value, is_secret)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&id)
    .bind(user)
    .bind(var.plugin)
    .bind(var.name)
    .bind(var.value)
    .bind(var.secret)
    .execute(pool)
    .await
    .expect("insert plugin env var");
    id
}

pub async fn count_rows(pool: &PgPool, sql: &'static str, arg: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .bind(arg)
        .fetch_one(pool)
        .await
        .expect("count rows")
}

// Writes `services/<rel>` under `dir`, creating parents.
pub fn write_services_file(dir: &std::path::Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create services subdirectory");
    }
    std::fs::write(path, contents).expect("write services file");
}
