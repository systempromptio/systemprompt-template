use anyhow::{anyhow, Result};
use serde_json::Value as JsonValue;

#[derive(Debug, Copy, Clone)]
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate a JSON value against a JSON schema
    pub fn validate(value: &JsonValue, schema: &JsonValue, strict: bool) -> Result<()> {
        Self::validate_value(value, schema, strict, "root")
    }

    fn validate_value(
        value: &JsonValue,
        schema: &JsonValue,
        strict: bool,
        path: &str,
    ) -> Result<()> {
        // Check type constraint
        if let Some(type_value) = schema.get("type") {
            Self::validate_type(value, type_value, path)?;
        }

        // Handle different schema types
        match schema.get("type").and_then(|t| t.as_str()) {
            Some("object") => Self::validate_object(value, schema, strict, path)?,
            Some("array") => Self::validate_array(value, schema, strict, path)?,
            Some("string") => Self::validate_string(value, schema, path)?,
            Some("number" | "integer") => Self::validate_number(value, schema, path)?,
            Some("boolean") => Self::validate_boolean(value, path)?,
            Some("null") => Self::validate_null(value, path)?,
            _ => {},
        }

        // Check enum constraint
        if let Some(enum_values) = schema.get("enum").and_then(|e| e.as_array()) {
            if !enum_values.contains(value) {
                return Err(anyhow!(
                    "Value at {path} must be one of: {enum_values:?}"
                ));
            }
        }

        Ok(())
    }

    fn validate_type(value: &JsonValue, type_schema: &JsonValue, path: &str) -> Result<()> {
        let valid_type = match type_schema {
            JsonValue::String(type_str) => Self::check_single_type(value, type_str),
            JsonValue::Array(types) => types.iter().any(|t| {
                t.as_str()
                    .is_some_and(|type_str| Self::check_single_type(value, type_str))
            }),
            _ => true,
        };

        if !valid_type {
            return Err(anyhow!(
                "Type mismatch at {}: expected {:?}, got {:?}",
                path,
                type_schema,
                Self::get_json_type(value)
            ));
        }

        Ok(())
    }

    fn check_single_type(value: &JsonValue, type_str: &str) -> bool {
        match type_str {
            "null" => value.is_null(),
            "boolean" => value.is_boolean(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "string" => value.is_string(),
            _ => true,
        }
    }

    fn validate_object(
        value: &JsonValue,
        schema: &JsonValue,
        strict: bool,
        path: &str,
    ) -> Result<()> {
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow!("{path} must be an object"))?;

        // Check required properties
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for req_prop in required {
                if let Some(prop_name) = req_prop.as_str() {
                    if !obj.contains_key(prop_name) {
                        return Err(anyhow!(
                            "Missing required property '{prop_name}' at {path}"
                        ));
                    }
                }
            }
        }

        // Validate properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in obj {
                let prop_path = format!("{path}.{key}");

                if let Some(prop_schema) = properties.get(key) {
                    Self::validate_value(value, prop_schema, strict, &prop_path)?;
                } else if strict {
                    // Check additionalProperties
                    let allow_additional = schema
                        .get("additionalProperties")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true);

                    if !allow_additional {
                        return Err(anyhow!("Unexpected property '{key}' at {path}"));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_array(
        value: &JsonValue,
        schema: &JsonValue,
        strict: bool,
        path: &str,
    ) -> Result<()> {
        let arr = value
            .as_array()
            .ok_or_else(|| anyhow!("{path} must be an array"))?;

        // Check min/max items
        if let Some(min_items) = schema.get("minItems").and_then(serde_json::Value::as_u64) {
            if arr.len() < min_items as usize {
                return Err(anyhow!("{path} must have at least {min_items} items"));
            }
        }

        if let Some(max_items) = schema.get("maxItems").and_then(serde_json::Value::as_u64) {
            if arr.len() > max_items as usize {
                return Err(anyhow!("{path} must have at most {max_items} items"));
            }
        }

        // Validate items
        if let Some(items_schema) = schema.get("items") {
            for (idx, item) in arr.iter().enumerate() {
                let item_path = format!("{path}[{idx}]");
                Self::validate_value(item, items_schema, strict, &item_path)?;
            }
        }

        Ok(())
    }

    fn validate_string(value: &JsonValue, schema: &JsonValue, path: &str) -> Result<()> {
        let str_val = value
            .as_str()
            .ok_or_else(|| anyhow!("{path} must be a string"))?;

        // Check min/max length
        if let Some(min_length) = schema.get("minLength").and_then(serde_json::Value::as_u64) {
            if str_val.len() < min_length as usize {
                return Err(anyhow!(
                    "{path} must have at least {min_length} characters"
                ));
            }
        }

        if let Some(max_length) = schema.get("maxLength").and_then(serde_json::Value::as_u64) {
            if str_val.len() > max_length as usize {
                return Err(anyhow!(
                    "{path} must have at most {max_length} characters"
                ));
            }
        }

        // Check pattern
        if let Some(pattern) = schema.get("pattern").and_then(|p| p.as_str()) {
            let re = regex::Regex::new(pattern)?;
            if !re.is_match(str_val) {
                return Err(anyhow!("{path} does not match pattern: {pattern}"));
            }
        }

        Ok(())
    }

    fn validate_number(value: &JsonValue, schema: &JsonValue, path: &str) -> Result<()> {
        let num_val = value
            .as_f64()
            .ok_or_else(|| anyhow!("{path} must be a number"))?;

        // Check minimum
        if let Some(minimum) = schema.get("minimum").and_then(serde_json::Value::as_f64) {
            if num_val < minimum {
                return Err(anyhow!("{path} must be >= {minimum}"));
            }
        }

        // Check maximum
        if let Some(maximum) = schema.get("maximum").and_then(serde_json::Value::as_f64) {
            if num_val > maximum {
                return Err(anyhow!("{path} must be <= {maximum}"));
            }
        }

        Ok(())
    }

    fn validate_boolean(value: &JsonValue, path: &str) -> Result<()> {
        if !value.is_boolean() {
            return Err(anyhow!("{path} must be a boolean"));
        }
        Ok(())
    }

    fn validate_null(value: &JsonValue, path: &str) -> Result<()> {
        if !value.is_null() {
            return Err(anyhow!("{path} must be null"));
        }
        Ok(())
    }

    const fn get_json_type(value: &JsonValue) -> &'static str {
        match value {
            JsonValue::Null => "null",
            JsonValue::Bool(_) => "boolean",
            JsonValue::Number(_) => "number",
            JsonValue::String(_) => "string",
            JsonValue::Array(_) => "array",
            JsonValue::Object(_) => "object",
        }
    }
}
