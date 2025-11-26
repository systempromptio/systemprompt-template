use anyhow::{anyhow, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

#[derive(Debug)]
pub struct SubjectAnalysisRepository {
    db_pool: DbPool,
}

impl SubjectAnalysisRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn analyze_conversation(
        &self,
        task_id: &str,
        keywords: &[String],
        primary_topic: &str,
        confidence: f64,
    ) -> Result<u64> {
        let keywords_json = serde_json::to_string(keywords)?;
        let query = DatabaseQueryEnum::AnalyzeConversation.get(self.db_pool.as_ref());
        self.db_pool
            .execute(
                &query,
                &[&task_id, &keywords_json, &primary_topic, &confidence],
            )
            .await
            .map_err(|e| anyhow!("Failed to analyze conversation: {e}"))
    }

    pub async fn get_top_subjects(&self, days: i32) -> Result<Vec<TopicStats>> {
        let query = DatabaseQueryEnum::GetTopSubjects.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .fetch_all(&query, &[&days])
            .await
            .map_err(|e| anyhow!("Failed to fetch top subjects: {e}"))?;

        Ok(rows
            .iter()
            .map(|r| TopicStats {
                topic: r
                    .get("primary_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                count: r.get("topic_count").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
                avg_confidence: r
                    .get("avg_confidence")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0),
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct TopicStats {
    pub topic: String,
    pub count: i32,
    pub avg_confidence: f64,
}
