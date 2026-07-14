use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId};
use systemprompt_web_shared::models::Content;

#[derive(Debug, Clone)]
pub(super) struct ContentQueryRepository {
    pool: Arc<PgPool>,
}

impl ContentQueryRepository {
    #[must_use]
    pub(super) const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub(super) async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            WHERE mc.id = $1
            "#,
            id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub(super) async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            WHERE mc.slug = $1
            "#,
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub(super) async fn get_by_source_and_slug(
        &self,
        source_id: &SourceId,
        slug: &str,
    ) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            WHERE mc.source_id = $1 AND mc.slug = $2
            "#,
            source_id.as_str(),
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub(super) async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            ORDER BY mc.published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub(super) async fn list_by_source(
        &self,
        source_id: &SourceId,
    ) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            WHERE mc.source_id = $1
            ORDER BY mc.published_at DESC
            "#,
            source_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub(super) async fn list_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT mc.id as "id!: ContentId", mc.slug as "slug!", mc.title as "title!", mc.description as "description!", mc.body as "body!", mc.author as "author!",
                   mc.published_at as "published_at!", mc.keywords as "keywords!", mc.kind as "kind!", mc.image,
                   mc.category_id as "category_id: CategoryId",
                   mc.source_id as "source_id!: SourceId",
                   mc.version_hash as "version_hash!",
                   COALESCE(mc.links, '[]'::jsonb) as "links!",
                   COALESCE(mce.after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(mce.related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(mce.related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(mce.related_docs, '[]'::jsonb) as "related_docs!",
                   mc.updated_at
            FROM markdown_content mc
            LEFT JOIN markdown_content_enrichment mce ON mce.content_id = mc.id
            ORDER BY mc.published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub(super) async fn get_slugs_by_source(
        &self,
        source_id: &SourceId,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar!(
            r#"SELECT slug FROM markdown_content WHERE source_id = $1"#,
            source_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }
}
