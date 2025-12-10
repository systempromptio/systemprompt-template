use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug)]
pub struct SubjectAnalysisRepository {
    pool: Arc<PgPool>,
}

impl SubjectAnalysisRepository {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn analyze_conversation(
        &self,
        task_id: &str,
        keywords: &[String],
        primary_topic: &str,
        confidence: f64,
    ) -> Result<u64> {
        let keywords_json = serde_json::to_string(keywords)?;

        let result = sqlx::query!(
            r#"
            INSERT INTO conversation_subjects (task_id, extracted_keywords, primary_topic, topic_confidence, analyzed_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (task_id) DO UPDATE SET
                extracted_keywords = $2,
                primary_topic = $3,
                topic_confidence = $4,
                analyzed_at = NOW()
            "#,
            task_id,
            keywords_json,
            primary_topic,
            confidence
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_top_subjects(&self, days: i32) -> Result<Vec<TopicStats>> {
        let rows = sqlx::query_as!(
            TopicStats,
            r#"
            SELECT
                primary_topic,
                COUNT(*)::int as topic_count,
                AVG(topic_confidence) as avg_confidence
            FROM conversation_subjects
            WHERE analyzed_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY primary_topic
            ORDER BY topic_count DESC
            LIMIT 20
            "#,
            days.to_string()
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TopicStats {
    pub primary_topic: Option<String>,
    pub topic_count: Option<i32>,
    pub avg_confidence: Option<f64>,
}
