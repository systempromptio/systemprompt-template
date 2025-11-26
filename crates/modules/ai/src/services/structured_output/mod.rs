pub mod parser;
pub mod validator;

use crate::models::ai::{ResponseFormat, StructuredOutputOptions};
use anyhow::{anyhow, Result};
use serde_json::Value as JsonValue;

#[derive(Debug, Copy, Clone)]
pub struct StructuredOutputProcessor;

impl StructuredOutputProcessor {
    /// Process and validate a response according to the specified format
    pub fn process_response(
        content: &str,
        format: &ResponseFormat,
        options: &StructuredOutputOptions,
    ) -> Result<JsonValue> {
        // First, try to extract JSON from the content
        let json_value =
            parser::JsonParser::extract_json(content, options.extraction_pattern.as_deref())?;

        // If we have a schema, validate against it
        if let ResponseFormat::JsonSchema { schema, strict, .. } = format {
            if options.validate_schema.unwrap_or(true) {
                let is_strict = strict.unwrap_or(true);
                validator::SchemaValidator::validate(&json_value, schema, is_strict)?;
            }
        }

        Ok(json_value)
    }

    /// Enhance a prompt to encourage JSON output
    pub fn enhance_prompt_for_json(
        original_prompt: &str,
        format: &ResponseFormat,
        options: &StructuredOutputOptions,
    ) -> String {
        if !options.inject_json_prompt.unwrap_or(true) {
            return original_prompt.to_string();
        }

        match format {
            ResponseFormat::Text => original_prompt.to_string(),
            ResponseFormat::JsonObject => {
                format!(
                    "{original_prompt}\n\nIMPORTANT: You must respond with valid JSON only. \
                    Do not include any text before or after the JSON object. \
                    Your entire response must be a valid JSON object."
                )
            },
            ResponseFormat::JsonSchema { schema, name, .. } => {
                let schema_str =
                    serde_json::to_string_pretty(schema).unwrap_or_else(|_| "{}".to_string());

                let schema_name = name.as_deref().unwrap_or("response");

                format!(
                    "{original_prompt}\n\nIMPORTANT: You must respond with valid JSON that conforms to this schema:\n\
                    Schema Name: {schema_name}\n\
                    ```json\n{schema_str}\n```\n\
                    Do not include any text before or after the JSON. \
                    Your entire response must be valid JSON matching this exact schema."
                )
            },
        }
    }

    /// Retry logic for structured output generation
    pub async fn generate_with_retry<F, Fut>(
        mut generator: F,
        format: &ResponseFormat,
        options: &StructuredOutputOptions,
    ) -> Result<JsonValue>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        let max_retries = options.max_retries.unwrap_or(3);
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match generator().await {
                Ok(content) => {
                    match Self::process_response(&content, format, options) {
                        Ok(json) => return Ok(json),
                        Err(e) => {
                            if attempt < max_retries {
                                // JSON parsing failed, will retry
                                last_error = Some(e);
                            } else {
                                return Err(e);
                            }
                        },
                    }
                },
                Err(e) => {
                    if attempt < max_retries {
                        // Generation failed, will retry
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                },
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Failed after {max_retries} retries")))
    }
}
