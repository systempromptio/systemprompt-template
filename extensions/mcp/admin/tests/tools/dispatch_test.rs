use systemprompt_admin::tools::{list_tools, register_tools};

#[test]
fn register_tools_returns_all_expected_tools() {
    let tools = register_tools();

    let expected_names = [
        "dashboard",
        "user",
        "traffic",
        "content",
        "conversations",
        "logs",
        "jobs",
        "operations",
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
    assert_eq!(tools.len(), 8);
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
