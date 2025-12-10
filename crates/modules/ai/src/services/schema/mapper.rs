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
