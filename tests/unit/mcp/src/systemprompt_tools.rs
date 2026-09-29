//! The `systemprompt` MCP server exposes a CLI passthrough tool, one read-only
//! dashboard report, and four typed admin-analytics tools. The passthrough's
//! schema is the whole contract: the model has to learn from the description
//! alone that the `systemprompt` prefix must be omitted, and `command` has to
//! be the one required argument or a call with no command reaches the CLI.
//! The output schema is the shared `ToolResponse<CliArtifact>` shape, so the
//! client can render the artifact rather than a blob of stdout.

use systemprompt_mcp_agent::tools::{
    CliInput, CliOutput, SERVER_NAME, input_schema, list_tools, output_schema,
};

// Why: this server exposes the CLI, ONE read-only report and four typed
// analytics tools, all over this platform's own data. It used to carry two
// Atlassian-backed tools as well; those are gone deliberately — the client
// reaches Jira and Confluence through the `atlassian` server it already
// holds, rather than through a proxy here that had to re-translate an
// unversioned payload shape.
#[test]
fn the_cli_the_report_and_the_typed_tools_are_the_whole_surface() {
    let tools = list_tools();
    assert_eq!(
        names(&tools),
        [
            "systemprompt",
            "admin_report",
            "user_activity",
            "conversation_list",
            "usage_by_user",
            "request_log",
            "conversation_audit",
            "users",
        ]
    );
    assert_eq!(tools[0].name.as_ref(), SERVER_NAME);
    assert_eq!(SERVER_NAME, "systemprompt");
    assert!(
        !names(&tools).iter().any(|n| n.contains("atlassian")
            || n.contains("project_catalog")
            || n.contains("jira")),
        "this server must not proxy Atlassian: {:?}",
        names(&tools)
    );
}

fn names(tools: &[rmcp::model::Tool]) -> Vec<String> {
    tools.iter().map(|t| t.name.to_string()).collect()
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
    for typed in [
        "user_activity",
        "conversation_list",
        "usage_by_user",
        "request_log",
        "conversation_audit",
        "users",
    ] {
        assert!(
            description.contains(typed),
            "the passthrough must point the model at `{typed}`"
        );
    }
    assert!(
        description.contains("--export"),
        "the description must say --export is stripped"
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

// Why: Claude Code validates `inputSchema.type == "object"` for every tool and
// drops the server's entire tool list when one fails. admin_report's tagged
// enum input shipped as a bare `oneOf` and took the CLI tool down with it.
#[test]
fn every_tool_input_schema_is_an_object_at_the_root() {
    for tool in list_tools() {
        assert_eq!(
            tool.input_schema.get("type"),
            Some(&serde_json::json!("object")),
            "{} must declare a root object input schema: {:?}",
            tool.name,
            tool.input_schema
        );
    }
}

// Why: Claude Code silently EXCLUDES a tool whose input schema it cannot
// accept, and says nothing the caller sees — `admin_report` was invisible to
// the CLI client for months because a serde-tagged enum emitted a top-level
// `oneOf` with no `properties`. The server kept advertising it and the model
// simply never had it. These are the client's documented acceptance rules,
// asserted here so a schema that would vanish fails the build instead.
//
// https://code.claude.com/docs/en/mcp — "Tool schema validation".
mod client_accepts_every_tool_schema {
    use super::list_tools;
    use serde_json::Value;

    // Rule 1: the schema root must be a plain object exposing `properties`.
    // The API rejects a root-level combinator; the client tries to flatten one
    // and drops the tool when the result does not validate. A plain object
    // never takes that path.
    #[test]
    fn every_input_schema_is_a_plain_object_with_properties() {
        for tool in list_tools() {
            let schema = Value::Object((*tool.input_schema).clone());
            let name = tool.name.as_ref();
            for combinator in ["oneOf", "anyOf", "allOf"] {
                assert!(
                    schema.get(combinator).is_none(),
                    "{name}: a root-level `{combinator}` is not accepted at the schema root. \
                     Use a flat object with a discriminator field and validate the combination \
                     in the handler — see ReportInput."
                );
            }
            assert_eq!(
                schema.get("type").and_then(Value::as_str),
                Some("object"),
                "{name}: the schema root must declare type=object"
            );
            assert!(
                schema.get("properties").is_some_and(Value::is_object),
                "{name}: the schema root must expose `properties`"
            );
        }
    }

    // Rule 2: every top-level property name is 1-64 characters of ASCII
    // letters, digits, underscore, dot or hyphen. A name outside that set
    // excludes the whole tool, not just the property.
    #[test]
    fn every_top_level_property_name_is_accepted() {
        for tool in list_tools() {
            let schema = Value::Object((*tool.input_schema).clone());
            let name = tool.name.as_ref();
            let properties = schema
                .get("properties")
                .and_then(Value::as_object)
                .expect("checked by the previous test");
            for key in properties.keys() {
                assert!(
                    (1..=64).contains(&key.len()),
                    "{name}: property `{key}` is {} characters; the limit is 1-64",
                    key.len()
                );
                assert!(
                    key.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)),
                    "{name}: property `{key}` uses characters outside [A-Za-z0-9_.-]"
                );
            }
        }
    }

    // Rule 3: a `$defs` block is only reachable through `$ref`, so a schema
    // that defines one must actually reference it. An orphaned definition is a
    // sign the root was rewritten and the reference lost — which is how a
    // discriminator enum silently becomes an untyped string.
    #[test]
    fn definitions_are_referenced_rather_than_orphaned() {
        for tool in list_tools() {
            let schema = Value::Object((*tool.input_schema).clone());
            let name = tool.name.as_ref();
            let Some(defs) = schema.get("$defs").and_then(Value::as_object) else {
                continue;
            };
            let rendered = schema.to_string();
            for key in defs.keys() {
                assert!(
                    rendered.contains(&format!("#/$defs/{key}")),
                    "{name}: `$defs/{key}` is defined but never referenced"
                );
            }
        }
    }
}
