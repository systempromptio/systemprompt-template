use serde_json::Value;

/// Trait for MCP servers that support artifact rendering.
///
/// Implement this trait to ensure your MCP server follows the correct pattern
/// for artifact schema handling. This trait enforces that servers:
/// 1. Can resolve output schemas for tools
/// 2. Validate that structured content has corresponding schemas
///
/// # Example
///
/// ```rust
/// use systemprompt_traits::ArtifactSupport;
///
/// impl ArtifactSupport for MyMcpServer {
///     fn get_output_schema_for_tool(
///         &self,
///         tool_name: &str,
///         arguments: &serde_json::Map<String, serde_json::Value>,
///     ) -> Option<serde_json::Value> {
///         match tool_name {
///             "introduce_yourself" => Some(tools::introduction_output_schema()),
///             "analyze_data" => arguments
///                 .get("type")
///                 .and_then(|v| v.as_str())
///                 .and_then(tools::get_analysis_schema),
///             _ => None,
///         }
///     }
/// }
/// ```
///
/// # Artifact Type Registry
///
/// The following `x-artifact-type` values are supported:
/// - `"presentation_card"` - Interactive cards with CTAs and sections
/// - `"table"` - Tabular data display
/// - `"chart"` - Data visualization (line, bar, pie charts)
/// - `"code"` - Syntax-highlighted code snippets
/// - `"markdown"` - Rich text content
///
/// # Schema Format
///
/// Output schemas MUST include the `x-artifact-type` metadata field:
///
/// ```json
/// {
///     "type": "object",
///     "x-artifact-type": "presentation_card",
///     "x-presentation-hints": {
///         "theme": "gradient"
///     },
///     "properties": {
///         "title": {"type": "string"},
///         "sections": {"type": "array"}
///     }
/// }
/// ```
pub trait ArtifactSupport {
    /// Resolves the output schema for a given tool based on its name and runtime arguments.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool being invoked
    /// * `arguments` - The arguments passed to the tool (may be used for dynamic schema selection)
    ///
    /// # Returns
    ///
    /// - `Some(Value)` - The JSON schema with `x-artifact-type` metadata
    /// - `None` - If the tool doesn't produce artifacts
    ///
    /// # Important
    ///
    /// - This method MUST return a schema for any tool that returns `structured_content`
    /// - The schema MUST include an `x-artifact-type` field
    /// - Without a schema, the frontend CANNOT render artifacts
    fn get_output_schema_for_tool(
        &self,
        tool_name: &str,
        arguments: &serde_json::Map<String, Value>,
    ) -> Option<Value>;

    /// Validates that a tool with structured output has a corresponding schema.
    ///
    /// This is a default implementation that returns validation status.
    /// Override this method if you need custom validation behavior.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool
    /// * `has_output` - Whether the tool returned structured content
    /// * `has_schema` - Whether an output schema was provided
    ///
    /// # Returns
    ///
    /// `true` if validation passes (schema present when output exists), `false` otherwise
    fn validate_artifact_schema(
        &self,
        _tool_name: &str,
        has_output: bool,
        has_schema: bool,
    ) -> bool {
        !has_output || has_schema
    }
}

/// Helper functions for creating common artifact schemas
pub mod schemas {
    use serde_json::{json, Value};

    /// Creates a presentation card schema with optional theme hints
    #[must_use]
    pub fn presentation_card(theme: Option<&str>) -> Value {
        let mut schema = json!({
            "type": "object",
            "x-artifact-type": "presentation_card",
            "properties": {
                "title": {"type": "string"},
                "subtitle": {"type": "string"},
                "sections": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": {"type": "string"},
                            "content": {"type": "string"},
                            "icon": {"type": "string"}
                        }
                    }
                },
                "ctas": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "label": {"type": "string"},
                            "message": {"type": "string"},
                            "variant": {"type": "string"},
                            "icon": {"type": "string"}
                        }
                    }
                },
                "theme": {"type": "string"}
            }
        });

        if let Some(theme_value) = theme {
            schema["x-presentation-hints"] = json!({"theme": theme_value});
        }

        schema
    }

    /// Creates a table schema for tabular data
    #[must_use]
    pub fn table() -> Value {
        json!({
            "type": "object",
            "x-artifact-type": "table",
            "properties": {
                "columns": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "rows": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                }
            },
            "required": ["columns", "rows"]
        })
    }

    /// Creates a chart schema with specified chart type
    #[must_use]
    pub fn chart(chart_type: &str) -> Value {
        json!({
            "type": "object",
            "x-artifact-type": "chart",
            "x-chart-type": chart_type,
            "properties": {
                "title": {"type": "string"},
                "data": {"type": "array"},
                "labels": {"type": "array"}
            }
        })
    }

    /// Creates a code snippet schema
    #[must_use]
    pub fn code(language: Option<&str>) -> Value {
        let mut schema = json!({
            "type": "object",
            "x-artifact-type": "code",
            "properties": {
                "code": {"type": "string"},
                "language": {"type": "string"}
            },
            "required": ["code"]
        });

        if let Some(lang) = language {
            schema["properties"]["language"]["default"] = json!(lang);
        }

        schema
    }

    /// Creates a markdown schema
    #[must_use]
    pub fn markdown() -> Value {
        json!({
            "type": "object",
            "x-artifact-type": "markdown",
            "properties": {
                "content": {"type": "string"}
            },
            "required": ["content"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestServer;

    impl ArtifactSupport for TestServer {
        fn get_output_schema_for_tool(
            &self,
            tool_name: &str,
            _arguments: &serde_json::Map<String, Value>,
        ) -> Option<Value> {
            match tool_name {
                "card_tool" => Some(schemas::presentation_card(Some("gradient"))),
                "table_tool" => Some(schemas::table()),
                _ => None,
            }
        }
    }

    #[test]
    fn test_schema_resolution() {
        let server = TestServer;
        let args = serde_json::Map::new();

        let card_schema = server.get_output_schema_for_tool("card_tool", &args);
        assert!(card_schema.is_some());
        assert_eq!(card_schema.unwrap()["x-artifact-type"], "presentation_card");

        let table_schema = server.get_output_schema_for_tool("table_tool", &args);
        assert!(table_schema.is_some());
        assert_eq!(table_schema.unwrap()["x-artifact-type"], "table");

        let no_schema = server.get_output_schema_for_tool("unknown_tool", &args);
        assert!(no_schema.is_none());
    }

    #[test]
    fn test_validation() {
        let server = TestServer;

        assert!(server.validate_artifact_schema("tool", true, true));
        assert!(server.validate_artifact_schema("tool", false, false));
        assert!(!server.validate_artifact_schema("tool", true, false));
        assert!(server.validate_artifact_schema("tool", false, true));
    }

    #[test]
    fn test_schema_helpers() {
        let card = schemas::presentation_card(Some("dark"));
        assert_eq!(card["x-artifact-type"], "presentation_card");
        assert_eq!(card["x-presentation-hints"]["theme"], "dark");

        let table = schemas::table();
        assert_eq!(table["x-artifact-type"], "table");

        let chart = schemas::chart("bar");
        assert_eq!(chart["x-artifact-type"], "chart");
        assert_eq!(chart["x-chart-type"], "bar");

        let code = schemas::code(Some("rust"));
        assert_eq!(code["x-artifact-type"], "code");

        let markdown = schemas::markdown();
        assert_eq!(markdown["x-artifact-type"], "markdown");
    }
}
