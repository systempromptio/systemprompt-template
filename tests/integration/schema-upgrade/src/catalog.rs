//! A normalised, sorted snapshot of a database's schema, one line per object,
//! so two databases can be diffed line by line.
//!
//! Names that neither path controls are dropped: constraint names (a
//! migration names its constraint, a baseline lets Postgres name it) and index
//! names. A `UNIQUE` constraint is compared through the unique index it owns
//! — a baseline declares `UNIQUE(a, b)`, a migration writes
//! `CREATE UNIQUE INDEX … (a, b)`, and the uniqueness is the same. `CHECK`
//! expressions are compared with casts and grouping stripped:
//! Postgres re-deparses an expression restored from dumped text with the
//! casts placed differently (`ARRAY[('x')::text]` against
//! `(ARRAY['x'])::text[]`) while meaning the same thing. Everything else —
//! column types, nullability, defaults, key definitions including
//! `NOT VALID`, index definitions, views, triggers, function bodies, enum
//! labels — is compared verbatim.

use sqlx::PgPool;

pub async fn snapshot(pool: &PgPool) -> Vec<String> {
    let mut lines = Vec::new();
    lines.extend(columns(pool).await);
    lines.extend(constraints(pool).await);
    lines.extend(indexes(pool).await);
    lines.extend(views(pool).await);
    lines.extend(triggers(pool).await);
    lines.extend(functions(pool).await);
    lines.extend(enums(pool).await);
    lines.sort();
    lines.dedup();
    lines
}

// Why: `- ` lines are present only in `before`, `+ ` lines only in `after`.
pub fn diff(before: &[String], after: &[String]) -> String {
    let mut out = String::new();
    for line in before {
        if after.binary_search(line).is_err() {
            out.push_str("  - ");
            out.push_str(line);
            out.push('\n');
        }
    }
    for line in after {
        if before.binary_search(line).is_err() {
            out.push_str("  + ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

async fn rows(pool: &PgPool, sql: &'static str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(sql)
        .fetch_all(pool)
        .await
        .expect("catalog query")
}

async fn columns(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('column %s.%s %s%s%s', c.relname, a.attname, \
                       format_type(a.atttypid, a.atttypmod), \
                       CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END, \
                       COALESCE(' DEFAULT ' || pg_get_expr(d.adbin, d.adrelid), '')) \
         FROM pg_attribute a \
         JOIN pg_class c ON c.oid = a.attrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
         WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p') \
           AND a.attnum > 0 AND NOT a.attisdropped",
    )
    .await
}

async fn constraints(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('constraint %s %s %s', c.conrelid::regclass, c.contype, \
                       CASE WHEN c.contype = 'c' \
                            THEN regexp_replace(regexp_replace(pg_get_constraintdef(c.oid), \
                                     '::[a-z ]+(\\[\\])?', '', 'g'), '[()\\[\\]]', '', 'g') \
                            ELSE pg_get_constraintdef(c.oid) END) \
         FROM pg_constraint c \
         JOIN pg_namespace n ON n.oid = c.connamespace \
         WHERE n.nspname = 'public' AND c.contype <> 'u'",
    )
    .await
}

async fn indexes(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('index %s %s', tablename, \
                       regexp_replace(indexdef, '^CREATE (UNIQUE )?INDEX \\S+ ON ', 'CREATE \\1INDEX ON ')) \
         FROM pg_indexes WHERE schemaname = 'public'",
    )
    .await
}

async fn views(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('view %s %s', c.relname, \
                       regexp_replace(pg_get_viewdef(c.oid, true), '\\s+', ' ', 'g')) \
         FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND c.relkind IN ('v', 'm')",
    )
    .await
}

async fn triggers(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('trigger %s', pg_get_triggerdef(t.oid)) \
         FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND NOT t.tgisinternal",
    )
    .await
}

async fn functions(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('function %s(%s) %s', p.proname, \
                       pg_get_function_identity_arguments(p.oid), md5(p.prosrc)) \
         FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace \
         WHERE n.nspname = 'public'",
    )
    .await
}

async fn enums(pool: &PgPool) -> Vec<String> {
    rows(
        pool,
        "SELECT format('enum %s %s', t.typname, \
                       string_agg(e.enumlabel, ',' ORDER BY e.enumsortorder)) \
         FROM pg_type t JOIN pg_enum e ON e.enumtypid = t.oid \
         JOIN pg_namespace n ON n.oid = t.typnamespace \
         WHERE n.nspname = 'public' GROUP BY t.typname",
    )
    .await
}
