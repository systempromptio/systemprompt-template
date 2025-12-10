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
