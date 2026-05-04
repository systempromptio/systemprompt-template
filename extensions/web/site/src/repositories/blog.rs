use sqlx::PgPool;

use crate::blog::types::{BlogPost, RelatedPost};

pub async fn list_blog_posts(pool: &PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
    sqlx::query_as!(
        BlogPost,
        r#"
        SELECT
            slug,
            title,
            description,
            image,
            category,
            published_at
        FROM markdown_content
        WHERE source_id = 'blog'
        ORDER BY published_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_related_posts(
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
