use crate::models::{Content, CreateContentParams};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId};

#[derive(Debug, Clone)]
pub struct ContentMutationRepository {
    pool: Arc<PgPool>,
}

impl ContentMutationRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateContentParams) -> Result<Content, sqlx::Error> {
        let id = ContentId::new(uuid::Uuid::new_v4().to_string());
        let now = Utc::now();
        sqlx::query_as!(
            Content,
            r#"
            INSERT INTO markdown_content (
                id, slug, title, description, body, author,
                published_at, keywords, kind, image, category_id, category, source_id,
                version_hash, links, after_reading_this, related_playbooks, related_code, related_docs, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            ON CONFLICT (slug) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                body = EXCLUDED.body,
                author = EXCLUDED.author,
                published_at = EXCLUDED.published_at,
                keywords = EXCLUDED.keywords,
                kind = EXCLUDED.kind,
                image = EXCLUDED.image,
                category_id = EXCLUDED.category_id,
                category = EXCLUDED.category,
                source_id = EXCLUDED.source_id,
                version_hash = EXCLUDED.version_hash,
                links = EXCLUDED.links,
                after_reading_this = EXCLUDED.after_reading_this,
                related_playbooks = EXCLUDED.related_playbooks,
                related_code = EXCLUDED.related_code,
                related_docs = EXCLUDED.related_docs,
                updated_at = EXCLUDED.updated_at
            RETURNING id as "id: ContentId", slug, title, description, body, author,
                      published_at, keywords, kind, image,
                      category_id as "category_id: CategoryId",
                      source_id as "source_id: SourceId",
                      version_hash,
                      COALESCE(links, '[]'::jsonb) as "links!",
                      COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                      COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                      COALESCE(related_code, '[]'::jsonb) as "related_code!",
                      COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                      updated_at
            "#,
            id.as_str(),
            params.slug,
            params.title,
            params.description,
            params.body,
            params.author,
            params.published_at,
            params.keywords,
            params.kind.as_str(),
            params.image,
            params.category_id.as_ref().map(CategoryId::as_str),
            params.category.as_deref(),
            params.source_id.as_str(),
            params.version_hash,
            params.links,
            params.after_reading_this,
            params.related_playbooks,
            params.related_code,
            params.related_docs,
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn update(&self, params: &UpdateContentParams) -> Result<Content, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as!(
            Content,
            r#"
            UPDATE markdown_content
            SET title = $1, description = $2, body = $3, keywords = $4,
                image = $5, version_hash = $6, updated_at = $7,
                links = COALESCE($9, links)
            WHERE id = $8
            RETURNING id as "id: ContentId", slug, title, description, body, author,
                      published_at, keywords, kind, image,
                      category_id as "category_id: CategoryId",
                      source_id as "source_id: SourceId",
                      version_hash,
                      COALESCE(links, '[]'::jsonb) as "links!",
                      COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                      COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                      COALESCE(related_code, '[]'::jsonb) as "related_code!",
                      COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                      updated_at
            "#,
            params.title,
            params.description,
            params.body,
            params.keywords,
            params.image,
            params.version_hash,
            now,
            params.id.as_str(),
            params.links
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn delete(&self, id: &ContentId) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM markdown_content WHERE id = $1", id.as_str())
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_orphaned_slugs(
        &self,
        source_id: &SourceId,
        found_slugs: &[String],
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"DELETE FROM markdown_content WHERE source_id = $1 AND slug != ALL($2)"#,
            source_id.as_str(),
            found_slugs
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone)]
pub struct UpdateContentParams {
    pub id: ContentId,
    pub title: String,
    pub description: String,
    pub body: String,
    pub keywords: String,
    pub image: Option<String>,
    pub version_hash: String,
    pub links: Option<serde_json::Value>,
}

impl UpdateContentParams {
    #[must_use]
    pub fn builder(
        id: ContentId,
        title: impl Into<String>,
        description: impl Into<String>,
        body: impl Into<String>,
        keywords: impl Into<String>,
        version_hash: impl Into<String>,
    ) -> UpdateContentParamsBuilder {
        UpdateContentParamsBuilder::new(id, title, description, body, keywords, version_hash)
    }
}

pub struct UpdateContentParamsBuilder {
    id: ContentId,
    title: String,
    description: String,
    body: String,
    keywords: String,
    version_hash: String,
    image: Option<String>,
    links: Option<serde_json::Value>,
}

impl UpdateContentParamsBuilder {
    #[must_use]
    pub fn new(
        id: ContentId,
        title: impl Into<String>,
        description: impl Into<String>,
        body: impl Into<String>,
        keywords: impl Into<String>,
        version_hash: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            body: body.into(),
            keywords: keywords.into(),
            version_hash: version_hash.into(),
            image: None,
            links: None,
        }
    }

    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    #[must_use]
    pub fn with_links(mut self, links: serde_json::Value) -> Self {
        self.links = Some(links);
        self
    }

    #[must_use]
    pub fn build(self) -> UpdateContentParams {
        UpdateContentParams {
            id: self.id,
            title: self.title,
            description: self.description,
            body: self.body,
            keywords: self.keywords,
            version_hash: self.version_hash,
            image: self.image,
            links: self.links,
        }
    }
}
