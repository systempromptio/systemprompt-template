use super::transformer::TransformedTool;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ToolNameMapper {
    forward_map: HashMap<String, (String, Option<String>, String)>,
    reverse_map: HashMap<String, Vec<String>>,
}

impl ToolNameMapper {
    pub fn new() -> Self {
        Self {
            forward_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn register_transformation(
        &mut self,
        transformed: &TransformedTool,
        discriminator_field: Option<String>,
    ) {
        let disc_field = discriminator_field.unwrap_or_else(|| "action".to_string());

        self.forward_map.insert(
            transformed.name.clone(),
            (
                transformed.original_name.clone(),
                transformed.discriminator_value.clone(),
                disc_field,
            ),
        );

        self.reverse_map
            .entry(transformed.original_name.clone())
            .or_default()
            .push(transformed.name.clone());
    }

    pub fn resolve_tool_call(&self, variant_name: &str, mut params: Value) -> (String, Value) {
        match self.forward_map.get(variant_name) {
            Some((original_name, Some(discriminator_value), discriminator_field)) => {
                if let Some(params_obj) = params.as_object_mut() {
                    params_obj.insert(
                        discriminator_field.clone(),
                        serde_json::json!(discriminator_value),
                    );
                }
                (original_name.clone(), params)
            },
            Some((original_name, None, _)) => (original_name.clone(), params),
            None => (variant_name.to_string(), params),
        }
    }

    pub fn get_variants(&self, original_name: &str) -> Option<&Vec<String>> {
        self.reverse_map.get(original_name)
    }

    pub fn is_variant(&self, tool_name: &str) -> bool {
        self.forward_map.contains_key(tool_name)
    }
}

impl Default for ToolNameMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_register_and_resolve() {
        let mut mapper = ToolNameMapper::new();

        let transformed = TransformedTool {
            name: "manage_agents_create".to_string(),
            description: "Create agent".to_string(),
            input_schema: json!({}),
            original_name: "manage_agents".to_string(),
            discriminator_value: Some("create".to_string()),
        };

        mapper.register_transformation(&transformed, Some("action".to_string()));

        let params = json!({"card": {"name": "test"}});
        let (original_name, resolved_params) =
            mapper.resolve_tool_call("manage_agents_create", params);

        assert_eq!(original_name, "manage_agents");
        assert_eq!(resolved_params["action"], "create");
        assert_eq!(resolved_params["card"]["name"], "test");
    }

    #[test]
    fn test_pass_through_tool() {
        let mut mapper = ToolNameMapper::new();

        let transformed = TransformedTool {
            name: "simple_tool".to_string(),
            description: "Simple".to_string(),
            input_schema: json!({}),
            original_name: "simple_tool".to_string(),
            discriminator_value: None,
        };

        mapper.register_transformation(&transformed, None);

        let params = json!({"name": "test"});
        let (original_name, resolved_params) =
            mapper.resolve_tool_call("simple_tool", params.clone());

        assert_eq!(original_name, "simple_tool");
        assert_eq!(resolved_params, params);
    }

    #[test]
    fn test_get_variants() {
        let mut mapper = ToolNameMapper::new();

        let create_tool = TransformedTool {
            name: "manage_agents_create".to_string(),
            description: "Create".to_string(),
            input_schema: json!({}),
            original_name: "manage_agents".to_string(),
            discriminator_value: Some("create".to_string()),
        };

        let read_tool = TransformedTool {
            name: "manage_agents_read".to_string(),
            description: "Read".to_string(),
            input_schema: json!({}),
            original_name: "manage_agents".to_string(),
            discriminator_value: Some("read".to_string()),
        };

        mapper.register_transformation(&create_tool, Some("action".to_string()));
        mapper.register_transformation(&read_tool, Some("action".to_string()));

        let variants = mapper.get_variants("manage_agents").unwrap();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"manage_agents_create".to_string()));
        assert!(variants.contains(&"manage_agents_read".to_string()));
    }

    #[test]
    fn test_is_variant() {
        let mut mapper = ToolNameMapper::new();

        let transformed = TransformedTool {
            name: "manage_agents_create".to_string(),
            description: "Create".to_string(),
            input_schema: json!({}),
            original_name: "manage_agents".to_string(),
            discriminator_value: Some("create".to_string()),
        };

        mapper.register_transformation(&transformed, Some("action".to_string()));

        assert!(mapper.is_variant("manage_agents_create"));
        assert!(!mapper.is_variant("unknown_tool"));
    }
}
