use sqlx::PgPool;

use crate::blog::types::{BlogPost, RelatedPost};

pub(crate) async fn list_blog_posts(pool: &PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
    sqlx::query_as!(
        BlogPost,
        r#"
        SELECT
            mc.slug,
            mc.title,
            mc.description,
            mc.image,
            mce.category,
            mc.published_at
        FROM markdown_content mc
        LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
        WHERE mc.source_id = 'blog'
        ORDER BY mc.published_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn list_related_posts(
    pool: &PgPool,
    current_slug: &str,
) -> Result<Vec<RelatedPost>, sqlx::Error> {
    sqlx::query_as!(
        RelatedPost,
        r#"
        SELECT slug, title
        FROM markdown_content
        WHERE source_id = 'blog'
        AND slug != $1
        ORDER BY published_at DESC
        LIMIT 3
        "#,
        current_slug
    )
    .fetch_all(pool)
    .await
}
