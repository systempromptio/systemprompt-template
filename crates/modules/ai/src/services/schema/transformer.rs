use super::analyzer::DiscriminatedUnion;
use super::capabilities::ProviderCapabilities;
use super::sanitizer::SchemaSanitizer;
use crate::models::tools::McpTool;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct TransformedTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub original_name: String,
    pub discriminator_value: Option<String>,
}

#[derive(Debug, Copy, Clone)]
pub struct SchemaTransformer {
    capabilities: ProviderCapabilities,
    sanitizer: SchemaSanitizer,
}

impl SchemaTransformer {
    pub const fn new(capabilities: ProviderCapabilities) -> Self {
        let sanitizer = SchemaSanitizer::new(capabilities);
        Self {
            capabilities,
            sanitizer,
        }
    }

    fn sanitize_function_name(name: &str) -> String {
        let mut result = String::new();

        for (i, ch) in name.chars().enumerate() {
            if i == 0 {
                if ch.is_alphabetic() || ch == '_' {
                    result.push(ch);
                } else if ch.is_numeric() {
                    result.push('_');
                    result.push(ch);
                } else {
                    result.push('_');
                }
            } else {
                match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | ':' | '-' => result.push(ch),
                    _ => result.push('_'),
                }
            }
        }

        if result.len() > 64 {
            result.truncate(64);
        }

        result
    }

    pub fn transform(&self, tool: &McpTool) -> Result<Vec<TransformedTool>> {
        let schema = tool
            .input_schema
            .as_ref()
            .ok_or_else(|| anyhow!("Tool '{}' missing required input_schema", tool.name))?;

        if !self.capabilities.requires_transformation(schema) {
            return Ok(vec![self.pass_through(tool)?]);
        }

        if let Some(union) = DiscriminatedUnion::detect(schema) {
            Ok(self.auto_split(tool, &union)?)
        } else {
            Ok(vec![self.pass_through(tool)?])
        }
    }

    fn pass_through(&self, tool: &McpTool) -> Result<TransformedTool> {
        let schema = tool
            .input_schema
            .as_ref()
            .ok_or_else(|| anyhow!("Tool '{}' missing required input_schema", tool.name))?
            .clone();

        let description = tool
            .description
            .as_ref()
            .filter(|d| !d.is_empty())
            .ok_or_else(|| anyhow!("Tool '{}' has empty or missing description", tool.name))?
            .clone();

        let sanitized_schema = self.sanitizer.sanitize(schema);

        Ok(TransformedTool {
            name: tool.name.clone(),
            description,
            input_schema: sanitized_schema,
            original_name: tool.name.clone(),
            discriminator_value: None,
        })
    }

    fn auto_split(
        &self,
        tool: &McpTool,
        union: &DiscriminatedUnion,
    ) -> Result<Vec<TransformedTool>> {
        let base_description = tool
            .description
            .as_ref()
            .filter(|d| !d.is_empty())
            .ok_or_else(|| anyhow!("Tool '{}' has empty or missing description", tool.name))?;

        let mut transformed_tools = Vec::new();

        for (variant_value, variant_schema) in &union.variants {
            let raw_name = format!("{}_{}", tool.name, variant_value);
            let variant_name = Self::sanitize_function_name(&raw_name);

            let mut merged_schema = json!({
                "type": "object",
                "properties": {}
            });

            if let Some(base_props) = union.base_properties.get("properties") {
                if let Some(base_obj) = base_props.as_object() {
                    if let Some(merged_props) = merged_schema.get_mut("properties") {
                        if let Some(merged_obj) = merged_props.as_object_mut() {
                            for (key, value) in base_obj {
                                if key != &union.discriminator_field {
                                    merged_obj.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }

            if let Some(variant_props) = variant_schema.get("properties") {
                if let Some(variant_obj) = variant_props.as_object() {
                    if let Some(merged_props) = merged_schema.get_mut("properties") {
                        if let Some(merged_obj) = merged_props.as_object_mut() {
                            for (key, value) in variant_obj {
                                merged_obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }

            // Merge required fields from both base and variant
            let mut all_required = Vec::new();

            // Add base required fields
            if let Some(base_required) = union.base_properties.get("required") {
                if let Some(base_arr) = base_required.as_array() {
                    for item in base_arr {
                        if let Some(field) = item.as_str() {
                            if field != union.discriminator_field {
                                all_required.push(json!(field));
                            }
                        }
                    }
                }
            }

            // Add variant required fields
            if let Some(variant_required) = variant_schema.get("required") {
                if let Some(variant_arr) = variant_required.as_array() {
                    for item in variant_arr {
                        if !all_required.contains(item) {
                            all_required.push(item.clone());
                        }
                    }
                }
            }

            if !all_required.is_empty() {
                merged_schema["required"] = json!(all_required);
            }

            let sanitized = self.sanitizer.sanitize(merged_schema);

            let description = format!(
                "{} - {}",
                base_description,
                Self::humanize_variant_name(variant_value)
            );

            transformed_tools.push(TransformedTool {
                name: variant_name,
                description,
                input_schema: sanitized,
                original_name: tool.name.clone(),
                discriminator_value: Some(variant_value.clone()),
            });
        }

        Ok(transformed_tools)
    }

    fn humanize_variant_name(variant: &str) -> String {
        variant
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_string()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_through() {
        let transformer = SchemaTransformer::new(ProviderCapabilities::anthropic());
        let tool = McpTool {
            name: "simple_tool".to_string(),
            description: Some("A simple tool".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            })),
            output_schema: None,
            service_id: "test".to_string(),
        };

        let transformed = transformer.transform(&tool).unwrap();
        assert_eq!(transformed.len(), 1);
        assert_eq!(transformed[0].name, "simple_tool");
        assert_eq!(transformed[0].original_name, "simple_tool");
        assert!(transformed[0].discriminator_value.is_none());
    }

    #[test]
    fn test_auto_split() {
        let transformer = SchemaTransformer::new(ProviderCapabilities::gemini());
        let tool = McpTool {
            name: "manage_agents".to_string(),
            description: Some("CRUD operations".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["create", "read"]
                    }
                },
                "required": ["action"],
                "allOf": [
                    {
                        "if": {
                            "properties": { "action": { "const": "create" } }
                        },
                        "then": {
                            "properties": {
                                "card": { "type": "object" }
                            },
                            "required": ["card"]
                        }
                    },
                    {
                        "if": {
                            "properties": { "action": { "const": "read" } }
                        },
                        "then": {
                            "properties": {
                                "agent_id": { "type": "string" }
                            },
                            "required": ["agent_id"]
                        }
                    }
                ]
            })),
            output_schema: None,
            service_id: "test".to_string(),
        };

        let transformed = transformer.transform(&tool).unwrap();
        assert_eq!(transformed.len(), 2);

        let create_tool = transformed
            .iter()
            .find(|t| t.name == "manage_agents_create")
            .unwrap();
        assert_eq!(create_tool.original_name, "manage_agents");
        assert_eq!(create_tool.discriminator_value, Some("create".to_string()));

        let props = create_tool
            .input_schema
            .get("properties")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(props.contains_key("card"));
        assert!(!props.contains_key("action"));
    }

    #[test]
    fn test_humanize_variant_name() {
        let transformer = SchemaTransformer::new(ProviderCapabilities::gemini());
        assert_eq!(transformer.humanize_variant_name("create"), "Create");
        assert_eq!(transformer.humanize_variant_name("read"), "Read");
        assert_eq!(transformer.humanize_variant_name("update"), "Update");
    }

    #[test]
    fn test_sanitize_function_name_valid() {
        assert_eq!(
            SchemaTransformer::sanitize_function_name("manage_agents"),
            "manage_agents"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("create_user_123"),
            "create_user_123"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool:name"),
            "tool:name"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool.name"),
            "tool.name"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool-name"),
            "tool-name"
        );
    }

    #[test]
    fn test_sanitize_function_name_starts_with_number() {
        assert_eq!(
            SchemaTransformer::sanitize_function_name("123create"),
            "_123create"
        );
        assert_eq!(SchemaTransformer::sanitize_function_name("9tool"), "_9tool");
    }

    #[test]
    fn test_sanitize_function_name_special_chars() {
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool@action"),
            "tool_action"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool#name!"),
            "tool_name_"
        );
        assert_eq!(
            SchemaTransformer::sanitize_function_name("tool$var%test"),
            "tool_var_test"
        );
    }

    #[test]
    fn test_sanitize_function_name_max_length() {
        let long_name = "a".repeat(70);
        let sanitized = SchemaTransformer::sanitize_function_name(&long_name);
        assert_eq!(sanitized.len(), 64);
        assert!(sanitized.starts_with('a'));
    }

    #[test]
    fn test_sanitize_function_name_underscore_prefix() {
        assert_eq!(SchemaTransformer::sanitize_function_name("_tool"), "_tool");
        assert_eq!(SchemaTransformer::sanitize_function_name("_123"), "__123");
    }
}
