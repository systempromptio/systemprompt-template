//! The `systemprompt` MCP server exposes exactly one tool, and its schema is
//! the whole contract: the model has to learn from the description alone that
//! the `systemprompt` prefix must be omitted, and `command` has to be the one
//! required argument or a call with no command reaches the CLI. The output
//! schema is the shared `ToolResponse<CliArtifact>` shape, so the client can
//! render the artifact rather than a blob of stdout.

use systemprompt_mcp_agent::tools::{
    CliInput, CliOutput, SERVER_NAME, input_schema, list_tools, output_schema,
};

#[test]
fn exactly_one_tool_is_exposed_under_the_server_name() {
    let tools = list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name.as_ref(), SERVER_NAME);
    assert_eq!(SERVER_NAME, "systemprompt");
}

#[test]
fn the_tool_carries_a_title_description_output_schema_and_ui_meta() {
    let tools = list_tools();
    let tool = &tools[0];

    assert_eq!(tool.title.as_deref(), Some("SystemPrompt CLI"));
    let description = tool.description.as_deref().expect("description is set");
    assert!(
        description.contains("WITHOUT the 'systemprompt' prefix"),
        "the prefix rule must be stated in the description; it is the only place the model sees it"
    );
    assert!(!tool.input_schema.is_empty());
    let output = tool.output_schema.as_ref().expect("output schema is set");
    assert!(!output.is_empty());
    assert!(
        tool.meta.is_some(),
        "UI meta is what attributes the call to this server"
    );
}

#[test]
fn command_is_the_single_required_input() {
    let schema = input_schema();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("the input schema declares required fields")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert_eq!(required, vec!["command"]);
    assert!(schema["properties"].get("command").is_some());
}

#[test]
fn the_listed_input_schema_is_the_one_the_tool_advertises() {
    let tools = list_tools();
    let listed = serde_json::Value::Object((*tools[0].input_schema).clone());

    assert_eq!(listed, input_schema());
    assert!(
        output_schema().is_object(),
        "the output schema must be a JSON Schema object"
    );
}

#[test]
fn cli_output_round_trips_the_fields_the_artifact_renders() {
    let payload = serde_json::json!({
        "stdout": "skill-a\nskill-b\n",
        "stderr": "",
        "exit_code": 0,
        "success": true,
    });

    let output: CliOutput = serde_json::from_value(payload.clone()).expect("cli output");
    assert!(output.success);
    assert_eq!(output.exit_code, 0);
    assert_eq!(serde_json::to_value(&output).expect("serializes"), payload);

    let input: CliInput =
        serde_json::from_value(serde_json::json!({ "command": "core skills list" }))
            .expect("cli input");
    assert_eq!(input.command, "core skills list");
    assert!(serde_json::from_value::<CliInput>(serde_json::json!({})).is_err());
}
