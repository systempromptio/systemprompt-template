//! Every tool the `systemprompt` MCP server lists must be declarable to
//! Gemini/Vertex. Core's checker states the rules the endpoint enforces (no
//! type lists, no `$ref`/`$defs`, arrays with `items`, typed composition
//! variants); a tool that breaks one costs the client the whole server's tool
//! list, not just that tool.

use serde_json::Value;
use systemprompt::models::schema::gemini_declaration_violations;
use systemprompt_mcp_agent::tools::list_tools;

#[test]
fn every_listed_tool_input_schema_is_gemini_declarable() {
    let tools = list_tools();
    assert!(!tools.is_empty());
    for tool in &tools {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        let violations = gemini_declaration_violations(&schema);
        assert!(
            violations.is_empty(),
            "tool `{}` is not Gemini-declarable: {violations:?}\n{schema:#}",
            tool.name
        );
    }
}

#[test]
fn report_days_is_a_plain_integer_with_a_default() {
    let tools = list_tools();
    let report = tools
        .iter()
        .find(|t| t.name.as_ref() == "admin_report")
        .expect("admin_report is listed");
    let days = &report.input_schema["properties"]["days"];
    assert_eq!(days["type"], "integer", "days: {days:#}");
    assert_eq!(days["default"], 7);
    assert!(days.get("anyOf").is_none() && days.get("oneOf").is_none());
    let kind = &report.input_schema["properties"]["report"];
    assert_eq!(
        kind["type"], "string",
        "report kind must be inlined: {kind:#}"
    );
    assert!(report.input_schema.get("$defs").is_none());
}

// Why: the typed tools exist so the model never has to guess a flag; a
// property that schemars rendered as a type list (an `Option`) or a `$ref`
// would make Gemini drop the whole server, and every string default is what
// lets a client omit the field rather than invent one.
#[test]
fn every_typed_tool_property_is_a_single_typed_scalar_with_a_default_or_required() {
    let tools = list_tools();
    for name in [
        "usage_by_user",
        "request_log",
        "conversation_audit",
        "users",
    ] {
        let tool = tools
            .iter()
            .find(|t| t.name.as_ref() == name)
            .unwrap_or_else(|| panic!("{name} is listed"));
        let schema = &tool.input_schema;
        assert!(schema.get("$defs").is_none(), "{name} has $defs");
        assert_eq!(
            schema["additionalProperties"], false,
            "{name} must deny unknown fields"
        );
        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let properties = schema["properties"].as_object().expect("properties");
        for (property, spec) in properties {
            assert!(
                spec["type"].is_string(),
                "{name}.{property} must be a single type: {spec:#}"
            );
            assert!(
                spec.get("anyOf").is_none() && spec.get("oneOf").is_none(),
                "{name}.{property} composes: {spec:#}"
            );
            assert!(
                spec.get("default").is_some() || required.contains(&property.as_str()),
                "{name}.{property} has neither a default nor is required: {spec:#}"
            );
        }
    }
}
