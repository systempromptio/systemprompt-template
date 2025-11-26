use crate::models::Skill;
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_identifiers::SkillId;

#[derive(Debug)]
pub struct SkillRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl SkillRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn create(&self, skill: &Skill) -> Result<()> {
        let query = DatabaseQueryEnum::CreateSkill.get(self.db.as_ref());

        let category_id_str = skill.category_id.as_ref().map(|c| c.as_str());

        self.db
            .execute(
                &query,
                &[
                    &skill.skill_id.as_str(),
                    &skill.file_path,
                    &skill.name,
                    &skill.description,
                    &skill.instructions,
                    &skill.enabled,
                    &skill.allowed_tools,
                    &skill.tags,
                    &category_id_str,
                    &skill.source_id.as_str(),
                ],
            )
            .await
            .context(format!("Failed to create skill: {}", skill.name))?;

        Ok(())
    }

    pub async fn get_by_skill_id(&self, skill_id: &SkillId) -> Result<Option<Skill>> {
        let query = DatabaseQueryEnum::GetSkillById.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&skill_id.as_str()])
            .await
            .context(format!("Failed to get skill by id: {}", skill_id))?;

        row.map(|r| Skill::from_json_row(&r)).transpose()
    }

    pub async fn get_by_file_path(&self, file_path: &str) -> Result<Option<Skill>> {
        let query = DatabaseQueryEnum::GetSkillByFilePath.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&file_path])
            .await
            .context(format!("Failed to get skill by file path: {}", file_path))?;

        row.map(|r| Skill::from_json_row(&r)).transpose()
    }

    pub async fn list_enabled(&self) -> Result<Vec<Skill>> {
        let query = DatabaseQueryEnum::ListEnabledSkills.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[])
            .await
            .context("Failed to list enabled skills")?;

        rows.iter()
            .map(|r| Skill::from_json_row(r))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn list_all(&self) -> Result<Vec<Skill>> {
        let query = DatabaseQueryEnum::ListAllSkills.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[])
            .await
            .context("Failed to list all skills")?;

        rows.iter()
            .map(|r| Skill::from_json_row(r))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn update(&self, skill_id: &SkillId, skill: &Skill) -> Result<()> {
        let query = DatabaseQueryEnum::UpdateSkill.get(self.db.as_ref());

        self.db
            .execute(
                &query,
                &[
                    &skill_id.as_str(),
                    &skill.name,
                    &skill.description,
                    &skill.instructions,
                    &skill.enabled,
                    &skill.allowed_tools,
                    &skill.tags,
                ],
            )
            .await
            .context(format!("Failed to update skill: {}", skill.name))?;

        Ok(())
    }
}
