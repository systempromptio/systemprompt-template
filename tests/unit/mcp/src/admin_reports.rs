//! Observable reporting invariants, including hostile input and incomplete
//! source data.
use serde_json::json;
use systemprompt::mcp::McpOutputSchema;
use systemprompt_mcp_agent::reports::{
    ReportInput, ReportOutput, normalize_cli, with_dollar_siblings,
};

#[test]
fn report_input_cannot_override_command_or_identity() {
    // `deny_unknown_fields` is what keeps a caller from smuggling a CLI
    // argument or an identity through the report input.
    for field in ["command", "profile", "database_url", "user_id"] {
        assert!(
            serde_json::from_value::<ReportInput>(json!({"report":"costs",field:"injected"}))
                .is_err(),
            "`{field}` must not deserialize into a report request"
        );
        assert!(
            serde_json::from_value::<ReportInput>(json!({"report":"costs","days":7,field:"x"}))
                .is_err(),
            "`{field}` must not deserialize alongside a valid request"
        );
    }
    // An unknown report kind is refused at the discriminator.
    // Every kind but `costs` is gone: Jira and Confluence are read by the
    // client directly, so a request naming them must be refused rather than
    // silently treated as a cost report.
    for gone in ["operations", "users", "projects", "brief", "activity"] {
        assert!(
            serde_json::from_value::<ReportInput>(json!({ "report": gone })).is_err(),
            "`{gone}` is not a report this server serves"
        );
    }
}

#[test]
fn days_defaults_to_seven_when_omitted() {
    let input = serde_json::from_value::<ReportInput>(json!({"report":"costs"}))
        .expect("a request without days is valid");
    assert_eq!(input.days, 7);
    let input = serde_json::from_value::<ReportInput>(json!({"report":"costs","days":30}))
        .expect("an explicit days is honoured");
    assert_eq!(input.days, 30);
    // A null is not "omitted": the field is a plain integer on the wire.
    assert!(serde_json::from_value::<ReportInput>(json!({"report":"costs","days":null})).is_err());
}

#[test]
fn stored_reports_from_before_request_and_highlights_are_still_renderable() {
    // These fields were added after reports had already been persisted. The
    // viewer must be able to open that historical data rather than dropping
    // the artifact because an optional display enhancement is absent.
    let report: ReportOutput = serde_json::from_value(json!({
        "report":"costs", "title":"AI Usage & Cost", "checked_at":"2026-09-01T00:00:00Z",
        "period":"2026-08-25T00:00:00Z to 2026-09-01T00:00:00Z", "complete":true,
        "sources":[], "metrics":{}, "tables":[]
    }))
    .expect("pre-addition report remains valid");

    assert!(report.highlights.is_empty());
    assert!(report.request.is_none());
    assert_eq!(ReportOutput::artifact_type(), "report");
    assert_eq!(report.artifact_title().as_deref(), Some("AI Usage & Cost"));
    assert!(
        report
            .text_body()
            .expect("report body")
            .contains("AI Usage & Cost")
    );
}

#[test]
fn report_data_renders_as_a_deterministic_mcp_asset() {
    use systemprompt::identifiers::{ArtifactId, ContextId};
    use systemprompt::mcp::services::ui_renderer::{RenderTarget, artifact_ui_resource};
    let data = json!({"report":"costs","title":"AI Cost & Adoption","checked_at":"2026-09-07T12:00:00Z",
        "period":"7 days","complete":true,"sources":[],"metrics":{},"tables":[],
        "request":{"report":"costs","days":7}});
    let html = artifact_ui_resource(&RenderTarget {
        artifact_id: &ArtifactId::generate(),
        artifact_type: "dashboard",
        payload: &data,
        context_id: ContextId::try_new("00000000-0000-4000-8000-000000000002")
            .expect("valid fixture identifier"),
        title: Some("Costs".into()),
    })
    .expect("render")
    .html;
    // The brand mark is written once, in title case, and uppercased by CSS —
    // asserting the rendered casing pinned a stylesheet detail rather than the
    // brand. "systemprompt.io" is the display name this template ships.
    assert!(html.contains("systemprompt.io"));
    // The page must not claim to show delivery health: its data is the audit
    // tables, and Jira never reaches it.
    assert!(html.contains("AI Usage"));
    assert!(!html.contains("traffic light"));
    assert!(html.contains("admin-read-"));
    assert!(html.contains("2026-09-07T12:00:00Z"));
    // Both placeholders must be substituted, or the page renders with no data
    // and no bridge and silently shows nothing.
    assert!(!html.contains("/*REPORT_DATA*/"));
    assert!(!html.contains("/*MCP_BRIDGE*/"));
}

#[test]
fn cost_chart_normalization_preserves_dollars_and_refuses_misaligned_data() {
    let chart = json!({"labels":["2026-09-07"],"datasets":[
        {"label":"cost_usd","data":[1.25]}, {"label":"requests","data":[4.0]}
    ]});
    let rows = normalize_cli(&chart).expect("chart");
    assert_eq!(rows[0]["cost_usd"], 1.25);
    assert_eq!(rows[0]["period"], "2026-09-07");
    assert!(
        normalize_cli(&json!({"labels":["today"],"datasets":[{"label":"cost_usd","data":[]}]}))
            .is_err()
    );
}

#[test]
fn report_normalization_accepts_each_documented_cli_envelope_without_rewriting_rows() {
    let item = json!({"heading":"total_requests","content":12});
    assert_eq!(
        normalize_cli(&json!({"items":[item.clone()]})).expect("items envelope"),
        vec![item.clone()]
    );
    assert_eq!(
        normalize_cli(&json!({"sections":[item.clone()]})).expect("sections envelope"),
        vec![item.clone()]
    );
    assert_eq!(
        normalize_cli(&json!([item.clone()])).expect("bare array"),
        vec![item.clone()]
    );
    assert_eq!(
        normalize_cli(&json!({"content": format!("[{item}]")})).expect("JSON text envelope"),
        vec![item]
    );
}

#[test]
fn report_normalization_refuses_text_that_is_not_json_and_unknown_envelopes() {
    let text = normalize_cli(&json!({"content":"human-readable summary"}))
        .expect_err("plain text cannot be presented as report rows");
    assert!(text.message.contains("text instead of structured"));

    let unknown = normalize_cli(&json!({"result":{"rows":[]}}))
        .expect_err("an unrecognised wrapper must not silently render an empty report");
    assert!(unknown.message.contains("Unsupported CLI report envelope"));
}

#[test]
fn report_normalization_refuses_charts_that_do_not_match_the_chart_contract() {
    // Having the marker fields selects the chart path. A malformed chart must
    // fail there rather than being mistaken for an arbitrary report object.
    let malformed = normalize_cli(&json!({"labels": "today", "datasets": []}))
        .expect_err("labels must be an array");
    assert!(malformed.message.contains("Malformed CLI chart"));
}

// Why: a renderer that cannot read its payload must SAY so. It used to fall
// through to core's DashboardRenderer, which cannot read a ReportOutput either
// — so the render failed, core dropped the embedded resource, and the tool
// returned success with no dashboard and no explanation. Silence looked
// identical to "this host does not support artifacts".
#[test]
fn a_payload_that_is_not_a_report_fails_loudly() {
    use systemprompt::identifiers::{ArtifactId, ContextId};
    use systemprompt::mcp::services::ui_renderer::{RenderTarget, artifact_ui_resource};

    let error = artifact_ui_resource(&RenderTarget {
        artifact_id: &ArtifactId::generate(),
        artifact_type: "dashboard",
        payload: &json!({"report": "costs", "tables": "not-an-array"}),
        context_id: ContextId::try_new("00000000-0000-4000-8000-000000000004")
            .expect("valid fixture identifier"),
        title: Some("Broken".into()),
    });

    assert!(
        error.is_err(),
        "a payload that does not match ReportOutput must not render as if it did"
    );
}

// Why: the model that read this report quoted "$12.11 millicents" from
// `avg_cost_per_request_microdollars`. Every microdollar cell must arrive
// with a `_usd` sibling so the converted figure is the one to quote, and the
// raw store stays beside it for reconciliation.
#[test]
fn every_microdollar_cell_gains_a_dollar_sibling() {
    let rows = with_dollar_siblings(vec![
        json!({"model":"claude-sonnet-5","total_cost_microdollars":15_150_809,"request_count":320}),
        json!({"heading":"total_cost_microdollars","content":30_569_751}),
        json!({"heading":"total_requests","content":2524}),
        json!("not an object"),
    ]);

    assert_eq!(rows[0]["total_cost_usd"], json!(15.150_809));
    assert_eq!(
        rows[0]["total_cost_microdollars"],
        json!(15_150_809),
        "the raw value is kept, not replaced"
    );
    assert!(
        rows[0].get("request_count_usd").is_none(),
        "only money is converted"
    );

    assert_eq!(rows[1]["heading"], json!("total_cost_microdollars"));
    assert_eq!(
        rows[2],
        json!({"heading":"total_cost_usd","content":30.569_751}),
        "a heading/content pair gains a sibling row, keeping the two-column shape"
    );
    assert_eq!(
        rows[3]["heading"],
        json!("total_requests"),
        "non-money pairs are untouched"
    );
    assert_eq!(
        rows[4],
        json!("not an object"),
        "non-object rows pass through"
    );
    assert_eq!(rows.len(), 5);
}
