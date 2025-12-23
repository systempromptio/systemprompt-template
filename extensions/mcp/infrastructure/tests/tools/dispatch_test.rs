use systemprompt_mcp_infrastructure::tools::{list_tools, register_tools};

#[test]
fn register_tools_returns_all_expected_tools() {
    let tools = register_tools();

    let expected_names = ["sync", "export", "deploy", "status", "config"];

    for name in &expected_names {
        assert!(
            tools.iter().any(|t| t.name.as_ref() == *name),
            "Missing tool: {}",
            name
        );
    }
}

#[test]
fn list_tools_returns_successful_result() {
    let result = list_tools();

    assert!(result.is_ok());
    let list_result = result.ok();
    assert!(list_result.is_some());
    let tools = list_result.map(|r| r.tools);
    assert!(tools.is_some());
    assert!(!tools.as_ref().map(|t| t.is_empty()).unwrap_or(true));
}

#[test]
fn all_tools_have_input_and_output_schemas() {
    let tools = register_tools();

    for tool in &tools {
        assert!(
            !tool.input_schema.is_empty(),
            "Tool {} missing input schema",
            tool.name
        );
        assert!(
            tool.output_schema.is_some(),
            "Tool {} missing output schema",
            tool.name
        );
    }
}

#[test]
fn register_tools_returns_correct_count() {
    let tools = register_tools();
    assert_eq!(tools.len(), 5);
}

#[test]
fn all_tools_have_descriptions() {
    let tools = register_tools();

    for tool in &tools {
        assert!(
            tool.description.is_some(),
            "Tool {} missing description",
            tool.name
        );
        let description = tool.description.as_ref().map(|d| d.as_ref());
        assert!(
            description.map(|d| !d.is_empty()).unwrap_or(false),
            "Tool {} has empty description",
            tool.name
        );
    }
}

#[test]
fn sync_tool_has_correct_schema() {
    let tools = register_tools();
    let sync = tools.iter().find(|t| t.name.as_ref() == "sync");

    assert!(sync.is_some(), "sync tool not found");
    let tool = sync.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("target").is_some());
    assert!(properties.get("direction").is_some());
    assert!(properties.get("dry_run").is_some());
}

#[test]
fn deploy_tool_has_correct_schema() {
    let tools = register_tools();
    let deploy = tools.iter().find(|t| t.name.as_ref() == "deploy");

    assert!(deploy.is_some(), "deploy tool not found");
    let tool = deploy.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("skip_build").is_some());
    assert!(properties.get("tag").is_some());
}

#[test]
fn export_tool_has_correct_schema() {
    let tools = register_tools();
    let export = tools.iter().find(|t| t.name.as_ref() == "export");

    assert!(export.is_some(), "export tool not found");
    let tool = export.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("target").is_some());
}

#[test]
fn status_tool_has_no_required_properties() {
    let tools = register_tools();
    let status = tools.iter().find(|t| t.name.as_ref() == "status");

    assert!(status.is_some(), "status tool not found");
    let tool = status.unwrap();

    let required = tool.input_schema.get("required");
    assert!(
        required.is_none()
            || required
                .unwrap()
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true)
    );
}

#[test]
fn config_tool_has_correct_schema() {
    let tools = register_tools();
    let config = tools.iter().find(|t| t.name.as_ref() == "config");

    assert!(config.is_some(), "config tool not found");
    let tool = config.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("filter").is_some());
    assert!(properties.get("format").is_some());
}
