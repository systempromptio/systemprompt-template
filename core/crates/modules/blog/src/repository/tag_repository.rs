use crate::models::Tag;
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_database::DatabaseQueryEnum;

#[derive(Debug)]
pub struct TagRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl TagRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn create(&self, tag: &Tag) -> Result<()> {
        let query = DatabaseQueryEnum::CreateTag.get(self.db.as_ref());

        self.db
            .execute(&query, &[&tag.id, &tag.name, &tag.slug])
            .await
            .context(format!("Failed to create tag: {}", tag.name))?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Tag>> {
        let query = DatabaseQueryEnum::ListTags.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[])
            .await
            .context("Failed to list tags")?;

        rows.iter()
            .map(Tag::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_or_create(&self, name: &str) -> Result<Tag> {
        let slug = Self::generate_slug(name);
        let id = uuid::Uuid::new_v4().to_string();

        let tag = Tag {
            id,
            name: name.to_string(),
            slug,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.create(&tag).await.ok();

        let query = DatabaseQueryEnum::GetTagByName.get(self.db.as_ref());
        let row = self
            .db
            .fetch_optional(&query, &[&name])
            .await
            .context(format!("Failed to get or create tag: {name}"))?;

        if let Some(row) = row {
            Tag::from_json_row(&row)
        } else {
            Ok(tag)
        }
    }

    pub async fn link_to_content(&self, tag_id: &str, content_id: &str) -> Result<()> {
        let query = DatabaseQueryEnum::LinkTagToContent.get(self.db.as_ref());

        self.db
            .execute(&query, &[&content_id, &tag_id])
            .await
            .context(format!(
                "Failed to link tag {tag_id} to content {content_id}"
            ))?;

        Ok(())
    }

    pub async fn unlink_all_from_content(&self, content_id: &str) -> Result<()> {
        let query = DatabaseQueryEnum::UnlinkAllTagsFromContent.get(self.db.as_ref());

        self.db
            .execute(&query, &[&content_id])
            .await
            .context(format!(
                "Failed to unlink all tags from content {content_id}"
            ))?;

        Ok(())
    }

    pub async fn get_by_content_id(&self, content_id: &str) -> Result<Vec<Tag>> {
        let query = DatabaseQueryEnum::GetTagsByContent.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&content_id])
            .await
            .context(format!("Failed to get tags for content {content_id}"))?;

        rows.iter()
            .map(Tag::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}
