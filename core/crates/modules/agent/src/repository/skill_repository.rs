use crate::models::{Skill, SkillRow};
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_identifiers::{CategoryId, SkillId, SourceId};

#[derive(Debug)]
pub struct SkillRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl SkillRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    fn get_pg_pool(&self) -> Result<Arc<PgPool>> {
        self.db
            .get_postgres_pool()
            .context("PostgreSQL pool not available")
    }

    pub async fn create(&self, skill: &Skill) -> Result<()> {
        let pool = self.get_pg_pool()?;
        let skill_id_str = skill.skill_id.as_str();
        let category_id = skill.category_id.as_ref().map(|c| c.as_str().to_string());
        let source_id_str = skill.source_id.as_str();

        sqlx::query!(
            "INSERT INTO agent_skills (skill_id, file_path, name, description, instructions,
             enabled, tags, category_id, source_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            skill_id_str,
            skill.file_path,
            skill.name,
            skill.description,
            skill.instructions,
            skill.enabled,
            &skill.tags[..],
            category_id,
            source_id_str
        )
        .execute(pool.as_ref())
        .await
        .context(format!("Failed to create skill: {}", skill.name))?;

        Ok(())
    }

    pub async fn get_by_skill_id(&self, skill_id: &SkillId) -> Result<Option<Skill>> {
        let pool = self.get_pg_pool()?;
        let skill_id_str = skill_id.as_str();

        let row = sqlx::query_as!(
            SkillRow,
            r#"SELECT
                skill_id as "skill_id!",
                file_path as "file_path!",
                name as "name!",
                description as "description!",
                instructions as "instructions!",
                enabled as "enabled!",
                tags,
                category_id,
                source_id as "source_id!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM agent_skills WHERE skill_id = $1"#,
            skill_id_str
        )
        .fetch_optional(pool.as_ref())
        .await
        .context(format!("Failed to get skill by id: {skill_id}"))?;

        row.map(skill_from_row).transpose()
    }

    pub async fn get_by_file_path(&self, file_path: &str) -> Result<Option<Skill>> {
        let pool = self.get_pg_pool()?;

        let row = sqlx::query_as!(
            SkillRow,
            r#"SELECT
                skill_id as "skill_id!",
                file_path as "file_path!",
                name as "name!",
                description as "description!",
                instructions as "instructions!",
                enabled as "enabled!",
                tags,
                category_id,
                source_id as "source_id!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM agent_skills WHERE file_path = $1"#,
            file_path
        )
        .fetch_optional(pool.as_ref())
        .await
        .context(format!("Failed to get skill by file path: {file_path}"))?;

        row.map(skill_from_row).transpose()
    }

    pub async fn list_enabled(&self) -> Result<Vec<Skill>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query_as!(
            SkillRow,
            r#"SELECT
                skill_id as "skill_id!",
                file_path as "file_path!",
                name as "name!",
                description as "description!",
                instructions as "instructions!",
                enabled as "enabled!",
                tags,
                category_id,
                source_id as "source_id!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM agent_skills WHERE enabled = true ORDER BY name ASC"#
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to list enabled skills")?;

        rows.into_iter()
            .map(skill_from_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn list_all(&self) -> Result<Vec<Skill>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query_as!(
            SkillRow,
            r#"SELECT
                skill_id as "skill_id!",
                file_path as "file_path!",
                name as "name!",
                description as "description!",
                instructions as "instructions!",
                enabled as "enabled!",
                tags,
                category_id,
                source_id as "source_id!",
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM agent_skills ORDER BY name ASC"#
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to list all skills")?;

        rows.into_iter()
            .map(skill_from_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn update(&self, skill_id: &SkillId, skill: &Skill) -> Result<()> {
        let pool = self.get_pg_pool()?;
        let skill_id_str = skill_id.as_str();

        sqlx::query!(
            "UPDATE agent_skills SET name = $1, description = $2, instructions = $3, enabled = $4,
             tags = $5, updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = $6",
            skill.name,
            skill.description,
            skill.instructions,
            skill.enabled,
            &skill.tags[..],
            skill_id_str
        )
        .execute(pool.as_ref())
        .await
        .context(format!("Failed to update skill: {}", skill.name))?;

        Ok(())
    }
}

fn skill_from_row(row: SkillRow) -> Result<Skill> {
    Ok(Skill {
        skill_id: SkillId::new(&row.skill_id),
        file_path: row.file_path,
        name: row.name,
        description: row.description,
        instructions: row.instructions,
        enabled: row.enabled,
        tags: row.tags.unwrap_or_default(),
        category_id: row.category_id.map(|c| CategoryId::new(&c)),
        source_id: SourceId::new(&row.source_id),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
