//! The boundary between the judge's database control flow and AI inference.

use crate::JobError;

use super::Classification;

/// The structured inference result consumed by judge persistence.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct JudgeVerdict {
    pub classification: Classification,
    pub ai_request_id: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

/// The AI inference boundary in a conversation-judge tick.
#[async_trait::async_trait]
#[doc(hidden)]
pub trait ConversationClassifier: Send + Sync {
    async fn classify(&self, transcript: &str) -> Result<JudgeVerdict, JobError>;
}
