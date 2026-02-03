use std::fmt::Write;
use std::sync::Arc;

use sqlx::PgPool;

use crate::error::SoulError;
use crate::models::{CreateMemoryParams, MemoryType, SoulMemory};
use crate::repository::MemoryRepository;

#[derive(Debug, Clone)]
pub struct MemoryService {
    repo: MemoryRepository,
}

impl MemoryService {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: MemoryRepository::new(pool),
        }
    }

    pub async fn store(&self, params: &CreateMemoryParams) -> Result<SoulMemory, SoulError> {
        self.repo.create(params).await.map_err(SoulError::from)
    }

    pub async fn get(&self, id: &str) -> Result<SoulMemory, SoulError> {
        self.repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| SoulError::MemoryNotFound(id.into()))
    }

    pub async fn recall(&self, id: &str) -> Result<SoulMemory, SoulError> {
        self.repo
            .recall(id)
            .await?
            .ok_or_else(|| SoulError::MemoryNotFound(id.into()))
    }

    pub async fn search(
        &self,
        query: &str,
        memory_type: Option<&str>,
        category: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<SoulMemory>, SoulError> {
        self.repo
            .search(query, memory_type, category, limit.unwrap_or(50))
            .await
            .map_err(SoulError::from)
    }

    pub async fn forget(&self, id: &str) -> Result<bool, SoulError> {
        self.repo.forget(id).await.map_err(SoulError::from)
    }

    pub async fn update(
        &self,
        id: &str,
        content: &str,
        context_text: Option<&str>,
    ) -> Result<bool, SoulError> {
        self.repo
            .update_content(id, content, context_text)
            .await
            .map_err(SoulError::from)
    }

    pub async fn get_context(
        &self,
        memory_types: Option<&[MemoryType]>,
        subject: Option<&str>,
        max_items: Option<i64>,
    ) -> Result<Vec<SoulMemory>, SoulError> {
        self.repo
            .get_context(memory_types, subject, max_items.unwrap_or(100))
            .await
            .map_err(SoulError::from)
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn build_context_string(
        &self,
        memory_types: Option<&[MemoryType]>,
        subject: Option<&str>,
        max_items: Option<i64>,
    ) -> Result<String, SoulError> {
        let memories = self.get_context(memory_types, subject, max_items).await?;

        if memories.is_empty() {
            return Ok("No memories stored yet.".to_string());
        }

        let mut output = String::from("## Memory Context\n\n");

        let mut current_type = String::new();
        for memory in &memories {
            if memory.memory_type != current_type {
                current_type.clone_from(&memory.memory_type);
                let type_label = match current_type.as_str() {
                    "core" => "Core",
                    "long_term" => "Long-term",
                    "short_term" => "Short-term",
                    "working" => "Working",
                    _ => &current_type,
                };
                writeln!(output, "### {type_label}").expect("write to String");
            }

            match &memory.context_text {
                Some(ctx) => writeln!(output, "- {ctx}"),
                None => writeln!(output, "- [{}] {}", memory.subject, memory.content),
            }
            .expect("write to String");
        }

        Ok(output)
    }

    pub async fn cleanup_expired(&self) -> Result<u64, SoulError> {
        self.repo.cleanup_expired().await.map_err(SoulError::from)
    }

    pub async fn count_active(&self) -> Result<i64, SoulError> {
        self.repo.count_active().await.map_err(SoulError::from)
    }

    pub async fn get_by_type(&self, memory_type: MemoryType) -> Result<Vec<SoulMemory>, SoulError> {
        self.repo
            .get_by_type(memory_type.as_str())
            .await
            .map_err(SoulError::from)
    }
}
