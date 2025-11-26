use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "json_object")]
    JsonObject,

    #[serde(rename = "json_schema")]
    JsonSchema {
        schema: JsonValue,
        name: Option<String>,
        strict: Option<bool>,
    },
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Text
    }
}

impl ResponseFormat {
    pub const fn json_object() -> Self {
        Self::JsonObject
    }

    pub const fn json_schema(schema: JsonValue) -> Self {
        Self::JsonSchema {
            schema,
            name: None,
            strict: Some(true),
        }
    }

    pub const fn json_schema_named(schema: JsonValue, name: String) -> Self {
        Self::JsonSchema {
            schema,
            name: Some(name),
            strict: Some(true),
        }
    }

    pub const fn is_json(&self) -> bool {
        !matches!(self, Self::Text)
    }

    pub const fn schema(&self) -> Option<&JsonValue> {
        match self {
            Self::JsonSchema { schema, .. } => Some(schema),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredOutputOptions {
    pub response_format: Option<ResponseFormat>,
    pub max_retries: Option<u8>,
    pub inject_json_prompt: Option<bool>,
    pub extraction_pattern: Option<String>,
    pub validate_schema: Option<bool>,
}

impl StructuredOutputOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_json_object() -> Self {
        Self {
            response_format: Some(ResponseFormat::JsonObject),
            inject_json_prompt: Some(true),
            validate_schema: Some(false),
            ..Default::default()
        }
    }

    pub fn with_schema(schema: JsonValue) -> Self {
        Self {
            response_format: Some(ResponseFormat::JsonSchema {
                schema,
                name: None,
                strict: Some(true),
            }),
            inject_json_prompt: Some(true),
            validate_schema: Some(true),
            max_retries: Some(3),
            ..Default::default()
        }
    }
}
