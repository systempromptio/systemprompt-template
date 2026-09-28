//! A host's namespaced MCP tool name reduced to the server and bare tool the
//! gateway records.

use systemprompt_web_admin::util::mcp_tool_name::{McpToolName, parse_mcp_tool_name_with};

fn known() -> Vec<String> {
    ["systemprompt", "google_workspace", "atlassian"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn plain_server_form_splits_on_the_last_separator() {
    assert_eq!(
        parse_mcp_tool_name_with("mcp__atlassian__listConfluenceContent", &known()),
        Some(McpToolName {
            server: "atlassian".to_owned(),
            tool: "listConfluenceContent".to_owned(),
        })
    );
}

#[test]
fn plugin_form_resolves_the_server_against_the_instance() {
    assert_eq!(
        parse_mcp_tool_name_with(
            "mcp__plugin_astound-super-admin_systemprompt__admin_report",
            &known()
        ),
        Some(McpToolName {
            server: "systemprompt".to_owned(),
            tool: "admin_report".to_owned(),
        })
    );
}

#[test]
fn plugin_form_keeps_an_underscored_server_id_whole() {
    assert_eq!(
        parse_mcp_tool_name_with(
            "mcp__plugin_astound-commons_google_workspace__search",
            &known()
        )
        .map(|n| n.server),
        Some("google_workspace".to_owned())
    );
}

#[test]
fn plugin_form_falls_back_to_the_last_segment_when_no_server_is_known() {
    assert_eq!(
        parse_mcp_tool_name_with("mcp__plugin_kit_other__run", &[]).map(|n| n.server),
        Some("other".to_owned())
    );
}

#[test]
fn builtin_tools_and_malformed_names_are_not_mcp() {
    assert_eq!(parse_mcp_tool_name_with("Write", &known()), None);
    assert_eq!(parse_mcp_tool_name_with("mcp__", &known()), None);
    assert_eq!(parse_mcp_tool_name_with("mcp____tool", &known()), None);
    assert_eq!(parse_mcp_tool_name_with("mcp__server__", &known()), None);
}
