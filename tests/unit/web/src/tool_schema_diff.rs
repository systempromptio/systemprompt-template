//! `types::tool_schema_diff` — the client and provider tool arrays of one
//! request read into one shape and paired by name. The three provider wire
//! shapes must all flatten to the same tool, a rewritten schema must be
//! flagged, a tool on one side only must be named rather than dropped, and
//! the Gemini rule hits must be computed only when the provider is Gemini.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use serde_json::json;
use systemprompt_web_admin::types::tool_schema_diff::{gemini_rules_apply, pair_tools, wire_tools};

fn schema() -> serde_json::Value {
    json!({"type": "object", "properties": {"q": {"type": "string"}}})
}

#[test]
fn anthropic_openai_and_gemini_shapes_flatten_to_the_same_tool() {
    let anthropic = json!([{"name": "search", "description": "Find", "input_schema": schema()}]);
    let openai = json!([{"type": "function", "function": {"name": "search", "description": "Find", "parameters": schema()}}]);
    let gemini = json!([{"function_declarations": [{"name": "search", "description": "Find", "parameters": schema()}]}]);

    let a = wire_tools(&anthropic);
    let o = wire_tools(&openai);
    let g = wire_tools(&gemini);
    assert_eq!(a, o);
    assert_eq!(a, g);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].name, "search");
    assert_eq!(a[0].description.as_deref(), Some("Find"));
    assert_eq!(a[0].schema, schema());
}

#[test]
fn a_gemini_entry_carries_several_declarations() {
    let gemini = json!([{"functionDeclarations": [
        {"name": "a", "parameters": schema()},
        {"name": "b", "parameters": schema()}
    ]}]);
    let names: Vec<String> = wire_tools(&gemini).into_iter().map(|t| t.name).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn a_non_array_or_nameless_entry_yields_nothing() {
    assert!(wire_tools(&json!({"name": "x"})).is_empty());
    assert!(wire_tools(&json!([{"input_schema": schema()}])).is_empty());
}

#[test]
fn an_identical_schema_is_as_sent_and_a_rewritten_one_is_flagged() {
    let client = wire_tools(&json!([
        {"name": "same", "input_schema": schema()},
        {"name": "fixed", "input_schema": {"type": "object", "properties": {"ids": {"type": "array"}}}}
    ]));
    let provider = wire_tools(&json!([{"function_declarations": [
        {"name": "same", "parameters": schema()},
        {"name": "fixed", "parameters": {"type": "object", "properties": {"ids": {"type": "array", "items": {"type": "string"}}}}}
    ]}]));
    let pairs = pair_tools(&client, &provider, true);
    assert_eq!(pairs.len(), 2);
    assert!(!pairs[0].changed);
    assert!(pairs[0].rule_hits.is_empty());
    assert!(pairs[1].changed);
    assert_eq!(pairs[1].rule_hits, ["$.ids: array without an items object"]);
}

#[test]
fn rule_hits_are_not_computed_for_a_non_gemini_provider() {
    let client = wire_tools(&json!([
        {"name": "t", "input_schema": {"type": "object", "properties": {"ids": {"type": "array"}}}}
    ]));
    let pairs = pair_tools(&client, &client, false);
    assert!(!pairs[0].changed);
    assert!(pairs[0].rule_hits.is_empty());
}

#[test]
fn tools_on_one_side_only_are_kept_and_named() {
    let client = wire_tools(&json!([{"name": "dropped", "input_schema": schema()}]));
    let provider = wire_tools(&json!([{"name": "added", "input_schema": schema()}]));
    let pairs = pair_tools(&client, &provider, false);
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].name, "dropped");
    assert!(pairs[0].client.is_some() && pairs[0].provider.is_none());
    assert!(!pairs[0].changed, "a tool never sent is not a rewrite");
    assert_eq!(pairs[1].name, "added");
    assert!(pairs[1].client.is_none() && pairs[1].provider.is_some());
    assert!(pairs[1].changed);
}

#[test]
fn gemini_rules_apply_to_gemini_and_vertex_only() {
    assert!(gemini_rules_apply("gemini"));
    assert!(gemini_rules_apply("Vertex"));
    assert!(gemini_rules_apply("google-vertex"));
    assert!(!gemini_rules_apply("anthropic"));
    assert!(!gemini_rules_apply("openai"));
    assert!(!gemini_rules_apply(""));
}
