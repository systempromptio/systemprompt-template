use systemprompt_infrastructure::tools::{list_tools, register_tools};

#[test]
fn register_tools_returns_all_expected_tools() {
    let tools = register_tools();

    let expected_names = [
        "sync_files",
        "sync_database",
        "deploy_crate",
        "sync_all",
        "sync_status",
    ];

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
fn sync_files_tool_has_correct_schema() {
    let tools = register_tools();
    let sync_files = tools.iter().find(|t| t.name.as_ref() == "sync_files");

    assert!(sync_files.is_some(), "sync_files tool not found");
    let tool = sync_files.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("direction").is_some());
    assert!(properties.get("dry_run").is_some());
}

#[test]
fn sync_database_tool_has_correct_schema() {
    let tools = register_tools();
    let sync_db = tools.iter().find(|t| t.name.as_ref() == "sync_database");

    assert!(sync_db.is_some(), "sync_database tool not found");
    let tool = sync_db.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("direction").is_some());
    assert!(properties.get("dry_run").is_some());
    assert!(properties.get("tables").is_some());
}

#[test]
fn deploy_crate_tool_has_correct_schema() {
    let tools = register_tools();
    let deploy = tools.iter().find(|t| t.name.as_ref() == "deploy_crate");

    assert!(deploy.is_some(), "deploy_crate tool not found");
    let tool = deploy.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("skip_build").is_some());
    assert!(properties.get("tag").is_some());
}

#[test]
fn sync_all_tool_has_correct_schema() {
    let tools = register_tools();
    let sync_all = tools.iter().find(|t| t.name.as_ref() == "sync_all");

    assert!(sync_all.is_some(), "sync_all tool not found");
    let tool = sync_all.unwrap();

    assert!(tool.input_schema.contains_key("properties"));
    let properties = tool.input_schema.get("properties").unwrap();

    assert!(properties.get("direction").is_some());
    assert!(properties.get("dry_run").is_some());
}

#[test]
fn sync_status_tool_has_empty_required_properties() {
    let tools = register_tools();
    let sync_status = tools.iter().find(|t| t.name.as_ref() == "sync_status");

    assert!(sync_status.is_some(), "sync_status tool not found");
    let tool = sync_status.unwrap();

    // sync_status has no required properties
    let required = tool.input_schema.get("required");
    assert!(
        required.is_none() || required.unwrap().as_array().map(|a| a.is_empty()).unwrap_or(true)
    );
}
