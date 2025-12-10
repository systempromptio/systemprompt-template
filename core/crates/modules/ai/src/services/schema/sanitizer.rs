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
        use serde_json::json;

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
            if !self.capabilities.supports_additional_properties {
                obj.remove("additionalProperties");
            }

            if !self.capabilities.supports_const {
                if let Some(const_val) = obj.remove("const") {
                    obj.insert("enum".to_string(), json!([const_val]));
                }
            }

            obj.remove("$schema");
            obj.remove("$id");

            obj.remove("readOnly");
            obj.remove("writeOnly");
            obj.remove("deprecated");
            obj.remove("examples");
            obj.remove("contentMediaType");
            obj.remove("contentEncoding");
            obj.remove("outputSchema");

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

            if let Some(any_of) = obj.get_mut("anyOf") {
                if let Some(arr) = any_of.as_array_mut() {
                    for item in arr.iter_mut() {
                        *item = self.sanitize(item.clone());
                    }
                }
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
