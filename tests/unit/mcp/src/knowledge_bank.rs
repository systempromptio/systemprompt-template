//! The knowledge-bank server's contract while its backend is a stub: three
//! tools mirroring the Node project-context server (search / list_sources /
//! index_stats) with `query` as search's only required argument, and a
//! `StubRetriever` that refuses to answer (`BACKEND_NOT_CONFIGURED`) rather
//! than fabricate results — an empty source list and not-configured errors
//! are the honest whole of what this scaffolding may claim.

use systemprompt::traits::ExtensionError;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::tools::{
    INDEX_STATS_TOOL, LIST_SOURCES_TOOL, SEARCH_TOOL, SERVER_NAME, list_tools, search_input_schema,
};
use systemprompt_mcp_knowledge_bank::{KnowledgeRetriever, SearchFilter, StubRetriever};

#[test]
fn three_tools_are_exposed_with_schemas_and_ui_meta() {
    let tools = list_tools();
    assert_eq!(tools.len(), 3);
    assert_eq!(SERVER_NAME, "knowledge-bank");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        vec![SEARCH_TOOL, LIST_SOURCES_TOOL, INDEX_STATS_TOOL]
    );

    for tool in &tools {
        assert!(tool.title.is_some());
        assert!(tool.description.is_some());
        assert!(!tool.input_schema.is_empty());
        let output = tool.output_schema.as_ref().expect("output schema is set");
        assert!(!output.is_empty());
        assert!(
            tool.meta.is_some(),
            "UI meta is what attributes the call to this server"
        );
    }
}

#[test]
fn search_schema_requires_query_with_optional_source_types_and_bounded_top_k() {
    let schema = search_input_schema();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(required, vec!["query"]);

    let source_types = &schema["properties"]["source_types"];
    assert!(source_types.is_object());

    // Why: schemars nests Option<u8> bounds differently across versions
    // (inline vs anyOf), so assert on the rendered property rather than a
    // hard-coded path.
    let top_k = serde_json::to_string(&schema["properties"]["top_k"]).expect("top_k serializes");
    assert!(top_k.contains("\"minimum\":1"));
    assert!(top_k.contains("\"maximum\":20"));
}

#[tokio::test]
async fn stub_search_reports_not_configured_instead_of_fake_data() {
    let filter = SearchFilter {
        source_types: vec!["confluence".to_owned()],
        top_k: Some(5),
    };
    let err = StubRetriever
        .search("deployment runbook", &filter)
        .await
        .expect_err("the stub must not return results");
    assert!(matches!(err, KnowledgeBankError::NotConfigured(_)));
    assert_eq!(err.code(), "BACKEND_NOT_CONFIGURED");
}

#[tokio::test]
async fn stub_lists_no_sources_and_has_no_index() {
    let sources = StubRetriever.list_sources().await.expect("listing works");
    assert!(sources.is_empty());

    let err = StubRetriever
        .index_stats()
        .await
        .expect_err("the stub has no index to report on");
    assert!(matches!(err, KnowledgeBankError::NotConfigured(_)));
}
