use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DiscriminatedUnion {
    pub discriminator_field: String,
    pub discriminator_values: Vec<String>,
    pub variants: HashMap<String, Value>,
    pub base_properties: Value,
}

impl DiscriminatedUnion {
    pub fn detect(schema: &Value) -> Option<Self> {
        let obj = schema.as_object()?;
        let all_of = obj.get("allOf")?.as_array()?;

        let mut discriminator_field: Option<String> = None;
        let mut variants = HashMap::new();
        let mut discriminator_values = Vec::new();

        for variant_schema in all_of {
            let variant_obj = variant_schema.as_object()?;
            let if_clause = variant_obj.get("if")?;
            let then_clause = variant_obj.get("then")?;

            let if_obj = if_clause.as_object()?;
            let if_props = if_obj.get("properties")?.as_object()?;

            let (field_name, field_value) = if_props.iter().next()?;

            if discriminator_field.is_none() {
                discriminator_field = Some(field_name.clone());
            } else if discriminator_field.as_ref() != Some(field_name) {
                return None;
            }

            let const_value = field_value.get("const")?.as_str()?;
            discriminator_values.push(const_value.to_string());
            variants.insert(const_value.to_string(), then_clause.clone());
        }

        let base_properties = obj
            .get("properties")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        Some(Self {
            discriminator_field: discriminator_field?,
            discriminator_values,
            variants,
            base_properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_discriminated_union() {
        let schema = json!({
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
        });

        let union = DiscriminatedUnion::detect(&schema).unwrap();

        assert_eq!(union.discriminator_field, "action");
        assert_eq!(union.discriminator_values, vec!["create", "read"]);
        assert_eq!(union.variants.len(), 2);
        assert!(union.variants.contains_key("create"));
        assert!(union.variants.contains_key("read"));
    }

    #[test]
    fn test_no_discriminated_union() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        assert!(DiscriminatedUnion::detect(&schema).is_none());
    }

    #[test]
    fn test_incomplete_pattern() {
        let schema = json!({
            "type": "object",
            "allOf": [
                {
                    "properties": { "name": { "type": "string" } }
                }
            ]
        });

        assert!(DiscriminatedUnion::detect(&schema).is_none());
    }
}
