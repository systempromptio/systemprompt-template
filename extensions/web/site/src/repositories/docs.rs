use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContentId, SourceId};
use systemprompt_web_shared::models::ContentLinkMetadata;

#[derive(Debug)]
pub(crate) struct DocContentRow {
    pub slug: String,
    pub kind: String,
    pub source_id: SourceId,
    pub after_reading_this: Json<Vec<String>>,
    pub related_playbooks: Json<Vec<ContentLinkMetadata>>,
    pub related_code: Json<Vec<ContentLinkMetadata>>,
}

pub(crate) async fn get_doc_content(
    pool: &PgPool,
    content_id: &ContentId,
) -> Result<DocContentRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            mc.slug,
            mc.kind,
            mc.source_id as "source_id: SourceId",
            COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!: Json<Vec<String>>",
            COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!: Json<Vec<ContentLinkMetadata>>",
            COALESCE(mce.related_code, '[]'::jsonb) as "related_code!: Json<Vec<ContentLinkMetadata>>"
        FROM markdown_content mc
        LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
        WHERE mc.id = $1
        "#,
        content_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(DocContentRow {
        slug: row.slug,
        kind: row.kind,
        source_id: row.source_id,
        after_reading_this: row.after_reading_this,
        related_playbooks: row.related_playbooks,
        related_code: row.related_code,
    })
}

#[derive(Debug)]
pub(crate) struct DocChildRow {
    pub slug: String,
    pub title: String,
    pub description: String,
}

pub(crate) async fn list_root_doc_children(
    pool: &PgPool,
    source_id: &SourceId,
) -> Result<Vec<DocChildRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT slug, title, description
        FROM markdown_content
        WHERE source_id = $1
          AND slug != ''
          AND slug != 'index'
          AND slug NOT LIKE '%/%'
        ORDER BY title
        "#,
        source_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| DocChildRow {
            slug: r.slug,
            title: r.title,
            description: r.description,
        })
        .collect())
}

pub(crate) async fn list_nested_doc_children(
    pool: &PgPool,
    source_id: &SourceId,
    slug_prefix: &str,
    current_slug: &str,
) -> Result<Vec<DocChildRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT slug, title, description
        FROM markdown_content
        WHERE source_id = $1
          AND slug LIKE $2
          AND slug != $3
        ORDER BY title
        "#,
        source_id.as_str(),
        slug_prefix,
        current_slug
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| DocChildRow {
            slug: r.slug,
            title: r.title,
            description: r.description,
        })
        .collect())
}
