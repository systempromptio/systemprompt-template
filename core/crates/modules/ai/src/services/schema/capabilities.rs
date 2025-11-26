use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProviderCapabilities {
    pub supports_allof: bool,
    pub supports_anyof: bool,
    pub supports_oneof: bool,
    pub supports_if_then_else: bool,
    pub supports_ref: bool,
    pub supports_definitions: bool,
    pub supports_not: bool,
}

impl ProviderCapabilities {
    pub const fn anthropic() -> Self {
        Self {
            supports_allof: true,
            supports_anyof: true,
            supports_oneof: true,
            supports_if_then_else: true,
            supports_ref: true,
            supports_definitions: true,
            supports_not: true,
        }
    }

    pub const fn openai() -> Self {
        Self {
            supports_allof: true,
            supports_anyof: true,
            supports_oneof: true,
            supports_if_then_else: false,
            supports_ref: true,
            supports_definitions: true,
            supports_not: false,
        }
    }

    pub const fn gemini() -> Self {
        Self {
            supports_allof: false,
            supports_anyof: false,
            supports_oneof: false,
            supports_if_then_else: false,
            supports_ref: false,
            supports_definitions: false,
            supports_not: false,
        }
    }

    pub fn requires_transformation(&self, schema: &Value) -> bool {
        if let Some(obj) = schema.as_object() {
            if obj.contains_key("allOf") && !self.supports_allof {
                return true;
            }
            if obj.contains_key("anyOf") && !self.supports_anyof {
                return true;
            }
            if obj.contains_key("oneOf") && !self.supports_oneof {
                return true;
            }
            if obj.contains_key("if") && !self.supports_if_then_else {
                return true;
            }
            if obj.contains_key("$ref") && !self.supports_ref {
                return true;
            }
            if (obj.contains_key("definitions") || obj.contains_key("$defs"))
                && !self.supports_definitions
            {
                return true;
            }
            if obj.contains_key("not") && !self.supports_not {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_anthropic_capabilities() {
        let caps = ProviderCapabilities::anthropic();
        assert!(caps.supports_allof);
        assert!(caps.supports_if_then_else);
        assert!(caps.supports_ref);
    }

    #[test]
    fn test_gemini_capabilities() {
        let caps = ProviderCapabilities::gemini();
        assert!(!caps.supports_allof);
        assert!(!caps.supports_if_then_else);
        assert!(!caps.supports_ref);
    }

    #[test]
    fn test_requires_transformation_gemini() {
        let caps = ProviderCapabilities::gemini();
        let schema = json!({
            "type": "object",
            "allOf": [{"properties": {}}]
        });
        assert!(caps.requires_transformation(&schema));
    }

    #[test]
    fn test_no_transformation_needed() {
        let caps = ProviderCapabilities::anthropic();
        let schema = json!({
            "type": "object",
            "properties": {"name": {"type": "string"}}
        });
        assert!(!caps.requires_transformation(&schema));
    }
}
