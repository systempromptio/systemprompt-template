use crate::models::Content;
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_database::DatabaseQueryEnum;

#[derive(Debug)]
pub struct ContentRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl ContentRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn create(&self, content: &Content) -> Result<()> {
        let query = DatabaseQueryEnum::CreateContent.get(self.db.as_ref());
        let links_json = serde_json::to_string(&content.links)?;

        self.db
            .execute(
                &query,
                &[
                    &content.id,
                    &content.slug,
                    &content.title,
                    &content.description,
                    &content.body,
                    &content.author,
                    &content.published_at,
                    &content.keywords,
                    &content.kind,
                    &content.image,
                    &content.category_id,
                    &content.source_id,
                    &content.version_hash,
                    &content.public,
                    &content.parent_content_id,
                    &links_json,
                ],
            )
            .await
            .context(format!("Failed to create content: {}", content.title))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Content>> {
        let query = DatabaseQueryEnum::GetContentById.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&id])
            .await
            .context(format!("Failed to get content by id: {id}"))?;

        row.map(|r| Content::from_json_row(&r)).transpose()
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>> {
        let query = DatabaseQueryEnum::GetContentByUrl.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&slug])
            .await
            .context(format!("Failed to get content by slug: {slug}"))?;

        row.map(|r| Content::from_json_row(&r)).transpose()
    }

    pub async fn get_by_source_and_slug(
        &self,
        source_id: &str,
        slug: &str,
    ) -> Result<Option<Content>> {
        let query = DatabaseQueryEnum::GetContentBySourceAndSlug.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&source_id, &slug])
            .await
            .context(format!(
                "Failed to get content by source {source_id} and slug: {slug}"
            ))?;

        row.map(|r| Content::from_json_row(&r)).transpose()
    }

    pub async fn list(&self, source_id: &str, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let query = DatabaseQueryEnum::ListContent.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&source_id, &limit, &offset])
            .await
            .context(format!(
                "Failed to list content for source: {source_id}, limit: {limit}, offset: {offset}"
            ))?;

        rows.iter()
            .map(Content::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>> {
        let query = DatabaseQueryEnum::ListAllContent.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&limit, &offset])
            .await
            .context(format!(
                "Failed to list all content with limit: {limit}, offset: {offset}"
            ))?;

        rows.iter()
            .map(Content::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn update(&self, id: &str, content: &Content) -> Result<()> {
        let query = DatabaseQueryEnum::UpdateContent.get(self.db.as_ref());
        let links_json = serde_json::to_string(&content.links)?;

        self.db
            .execute(
                &query,
                &[
                    &content.title,
                    &content.description,
                    &content.body,
                    &content.author,
                    &content.published_at,
                    &content.keywords,
                    &content.kind,
                    &content.image,
                    &content.category_id,
                    &content.source_id,
                    &content.version_hash,
                    &links_json,
                    &id,
                ],
            )
            .await
            .context(format!("Failed to update content: {id}"))?;

        Ok(())
    }

    pub async fn update_image(&self, id: &str, image_url: &str) -> Result<()> {
        let query = DatabaseQueryEnum::UpdateContentImage.get(self.db.as_ref());

        self.db
            .execute(&query, &[&image_url, &chrono::Utc::now(), &id])
            .await
            .context(format!("Failed to update image for content: {id}"))?;

        Ok(())
    }

    pub async fn get_social_content_by_parent(&self, parent_id: &str) -> Result<Vec<Content>> {
        let query = DatabaseQueryEnum::GetSocialContentByParent.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&parent_id])
            .await
            .context(format!(
                "Failed to get social content by parent: {parent_id}"
            ))?;

        rows.iter()
            .map(Content::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_by_version_hash(&self, version_hash: &str) -> Result<Option<Content>> {
        let query = DatabaseQueryEnum::GetContentByVersionHash.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&version_hash])
            .await
            .context(format!(
                "Failed to get content by version_hash: {version_hash}"
            ))?;

        row.map(|r| Content::from_json_row(&r)).transpose()
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let query = DatabaseQueryEnum::DeleteContent.get(self.db.as_ref());

        self.db
            .execute(&query, &[&id])
            .await
            .context(format!("Failed to delete content with id: {id}"))?;

        Ok(())
    }

    pub async fn delete_by_source(&self, source_id: &str) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteContentBySource.get(self.db.as_ref());

        self.db
            .execute(&query, &[&source_id])
            .await
            .context(format!(
                "Failed to delete content for source: {source_id}"
            ))
    }

    pub async fn list_by_source(&self, source_id: &str) -> Result<Vec<Content>> {
        let query = DatabaseQueryEnum::ListContentBySource.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&source_id])
            .await
            .context(format!("Failed to list content by source: {source_id}"))?;

        rows.iter()
            .map(Content::from_json_row)
            .collect::<Result<Vec<_>>>()
    }
}
