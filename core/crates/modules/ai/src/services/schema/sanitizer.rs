use super::capabilities::ProviderCapabilities;
use serde_json::Value;

#[derive(Debug, Copy, Clone)]
pub struct SchemaSanitizer {
    capabilities: ProviderCapabilities,
}

impl SchemaSanitizer {
    pub const fn new(capabilities: ProviderCapabilities) -> Self {
        Self { capabilities }
    }

    pub fn sanitize(&self, schema: Value) -> Value {
        let mut sanitized = schema;

        if let Some(obj) = sanitized.as_object_mut() {
            if !self.capabilities.supports_allof {
                obj.remove("allOf");
            }
            if !self.capabilities.supports_anyof {
                obj.remove("anyOf");
            }
            if !self.capabilities.supports_oneof {
                obj.remove("oneOf");
            }
            if !self.capabilities.supports_if_then_else {
                obj.remove("if");
                obj.remove("then");
                obj.remove("else");
            }
            if !self.capabilities.supports_ref {
                obj.remove("$ref");
            }
            if !self.capabilities.supports_definitions {
                obj.remove("definitions");
                obj.remove("$defs");
            }
            if !self.capabilities.supports_not {
                obj.remove("not");
            }

            obj.remove("$schema");
            obj.remove("$id");

            // Remove fields not supported by Gemini
            obj.remove("readOnly");
            obj.remove("writeOnly");
            obj.remove("deprecated");
            obj.remove("examples");
            obj.remove("contentMediaType");
            obj.remove("contentEncoding");

            let keys_to_remove: Vec<String> = obj
                .keys()
                .filter(|k| k.starts_with("x-"))
                .cloned()
                .collect();
            for key in keys_to_remove {
                obj.remove(&key);
            }

            if let Some(properties) = obj.get_mut("properties") {
                if let Some(props_obj) = properties.as_object_mut() {
                    for (_key, value) in props_obj.iter_mut() {
                        *value = self.sanitize(value.clone());
                    }
                }
            }

            if let Some(items) = obj.get_mut("items") {
                *items = self.sanitize(items.clone());
            }

            if let Some(additional_props) = obj.get_mut("additionalProperties") {
                if additional_props.is_object() {
                    *additional_props = self.sanitize(additional_props.clone());
                }
            }
        }

        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_gemini() {
        let sanitizer = SchemaSanitizer::new(ProviderCapabilities::gemini());
        let schema = json!({
            "type": "object",
            "allOf": [{"type": "object"}],
            "$schema": "http://json-schema.org/draft-07/schema#",
            "properties": {
                "name": { "type": "string" }
            }
        });

        let sanitized = sanitizer.sanitize(schema);
        let obj = sanitized.as_object().unwrap();

        assert!(!obj.contains_key("allOf"));
        assert!(!obj.contains_key("$schema"));
        assert!(obj.contains_key("properties"));
    }

    #[test]
    fn test_sanitize_nested() {
        let sanitizer = SchemaSanitizer::new(ProviderCapabilities::gemini());
        let schema = json!({
            "type": "object",
            "properties": {
                "nested": {
                    "type": "object",
                    "allOf": [{"type": "object"}],
                    "properties": {
                        "inner": { "type": "string" }
                    }
                }
            }
        });

        let sanitized = sanitizer.sanitize(schema);
        let nested = &sanitized["properties"]["nested"];
        let nested_obj = nested.as_object().unwrap();

        assert!(!nested_obj.contains_key("allOf"));
        assert!(nested_obj.contains_key("properties"));
    }

    #[test]
    fn test_anthropic_no_sanitization() {
        let sanitizer = SchemaSanitizer::new(ProviderCapabilities::anthropic());
        let schema = json!({
            "type": "object",
            "allOf": [{"type": "object"}],
            "properties": {
                "name": { "type": "string" }
            }
        });

        let sanitized = sanitizer.sanitize(schema);
        let obj = sanitized.as_object().unwrap();

        assert!(obj.contains_key("allOf"));
    }

    #[test]
    fn test_remove_nested_vendor_extensions() {
        let sanitizer = SchemaSanitizer::new(ProviderCapabilities::gemini());
        let schema = json!({
            "type": "object",
            "x-top-level": "should be removed",
            "properties": {
                "uuid": {
                    "type": "string",
                    "description": "Agent UUID",
                    "x-data-source": {
                        "tool": "manage_agents",
                        "action": "read"
                    }
                },
                "nested": {
                    "type": "object",
                    "x-nested-vendor": "test",
                    "properties": {
                        "deep": {
                            "type": "string",
                            "x-deep-vendor": "should also be removed"
                        }
                    }
                }
            }
        });

        let sanitized = sanitizer.sanitize(schema);
        let obj = sanitized.as_object().unwrap();

        // Top-level vendor extension should be removed
        assert!(!obj.contains_key("x-top-level"));

        // Nested vendor extensions should be removed
        let uuid_prop = &sanitized["properties"]["uuid"];
        assert!(!uuid_prop.as_object().unwrap().contains_key("x-data-source"));

        let nested_prop = &sanitized["properties"]["nested"];
        assert!(!nested_prop
            .as_object()
            .unwrap()
            .contains_key("x-nested-vendor"));

        let deep_prop = &sanitized["properties"]["nested"]["properties"]["deep"];
        assert!(!deep_prop.as_object().unwrap().contains_key("x-deep-vendor"));
    }
}
