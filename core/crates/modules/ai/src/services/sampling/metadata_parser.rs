use crate::models::ai::{ModelHint, ModelPreferences, SamplingMetadata};
use anyhow::Result;

#[derive(Debug, Copy, Clone)]
pub struct MetadataParser;

impl MetadataParser {
    pub fn parse_sampling_metadata(input: serde_json::Value) -> Result<SamplingMetadata> {
        Ok(serde_json::from_value(input)?)
    }

    pub fn parse_model_preferences(input: serde_json::Value) -> Result<ModelPreferences> {
        Ok(serde_json::from_value(input)?)
    }

    pub fn merge_metadata(
        base: &SamplingMetadata,
        override_metadata: Option<&SamplingMetadata>,
    ) -> SamplingMetadata {
        if let Some(override_metadata) = override_metadata {
            SamplingMetadata {
                temperature: override_metadata.temperature.or(base.temperature),
                top_p: override_metadata.top_p.or(base.top_p),
                top_k: override_metadata.top_k.or(base.top_k),
                presence_penalty: override_metadata.presence_penalty.or(base.presence_penalty),
                frequency_penalty: override_metadata
                    .frequency_penalty
                    .or(base.frequency_penalty),
                stop_sequences: override_metadata
                    .stop_sequences
                    .clone()
                    .or_else(|| base.stop_sequences.clone()),
                user_id: override_metadata
                    .user_id
                    .clone()
                    .or_else(|| base.user_id.clone()),
                session_id: override_metadata
                    .session_id
                    .clone()
                    .or_else(|| base.session_id.clone()),
                trace_id: override_metadata
                    .trace_id
                    .clone()
                    .or_else(|| base.trace_id.clone()),
                agent_id: override_metadata
                    .agent_id
                    .clone()
                    .or_else(|| base.agent_id.clone()),
                task_id: override_metadata
                    .task_id
                    .clone()
                    .or_else(|| base.task_id.clone()),
            }
        } else {
            base.clone()
        }
    }

    pub fn validate_metadata(metadata: &SamplingMetadata) -> Result<()> {
        if let Some(temp) = metadata.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(anyhow::anyhow!("Temperature must be between 0 and 2"));
            }
        }

        if let Some(top_p) = metadata.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(anyhow::anyhow!("Top-p must be between 0 and 1"));
            }
        }

        if let Some(top_k) = metadata.top_k {
            if top_k < 1 {
                return Err(anyhow::anyhow!("Top-k must be at least 1"));
            }
        }

        Ok(())
    }

    pub fn extract_model_hints(text: &str) -> Vec<ModelHint> {
        let mut hints = Vec::new();

        let text_lower = text.to_lowercase();

        if text_lower.contains("fast") || text_lower.contains("quick") {
            hints.push(ModelHint::Category("fast".to_string()));
        }

        if text_lower.contains("quality") || text_lower.contains("best") {
            hints.push(ModelHint::Category("quality".to_string()));
        }

        if text_lower.contains("claude") {
            hints.push(ModelHint::Provider("anthropic".to_string()));
        }

        if text_lower.contains("gpt") || text_lower.contains("openai") {
            hints.push(ModelHint::Provider("openai".to_string()));
        }

        if text_lower.contains("gemini") {
            hints.push(ModelHint::Provider("gemini".to_string()));
        }

        hints
    }
}
