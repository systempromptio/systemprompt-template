//! The passthrough result bound. `oversize_bytes` is the overflow decision:
//! a result under the bound is sent as is, one over it is stored whole as an
//! artifact and the model receives `overflow_pointer` naming where it is.
//! `bound_artifact` is the fallback when that store fails: a table loses
//! trailing rows and says so in its title, and any other oversized shape
//! becomes truncated text — never a spilled file.

use serde_json::json;
use systemprompt::identifiers::ArtifactId;
use systemprompt::models::artifacts::{
    CliArtifact, Column, ColumnType, TableArtifact, TextArtifact,
};
use systemprompt_mcp_agent::bounds::{
    MAX_RESULT_BYTES, StoredOverflow, bound_artifact, overflow_pointer, oversize_bytes, row_count,
    wire_bytes,
};

fn table(rows: usize, cell: &str) -> CliArtifact {
    let items = (0..rows)
        .map(|i| json!({"request_id": format!("req_{i}"), "content": cell}))
        .collect();
    CliArtifact::Table {
        artifact: TableArtifact::new(vec![
            Column::new("request_id", ColumnType::String),
            Column::new("content", ColumnType::String),
        ])
        .with_title("AI Requests")
        .with_rows(items),
    }
}

#[test]
fn a_small_table_passes_through_untouched() {
    let bounded = bound_artifact(table(5, "hello"), "infra logs request list");
    assert!(!bounded.truncated);
    assert!(bounded.rows.is_none());
    let CliArtifact::Table { artifact } = bounded.artifact else {
        panic!("shape must be preserved");
    };
    assert_eq!(artifact.items.len(), 5);
    assert_eq!(artifact.title.as_deref(), Some("AI Requests"));
}

#[test]
fn an_oversized_table_keeps_a_prefix_of_rows_and_names_the_cut() {
    let big = "x".repeat(1024);
    let bounded = bound_artifact(table(2000, &big), "infra logs request list -n 2000");
    assert!(bounded.truncated);
    let (kept, received) = bounded.rows.expect("row accounting");
    assert_eq!(received, 2000);
    assert!(kept > 0 && kept < 2000, "kept {kept}");
    let CliArtifact::Table { artifact } = bounded.artifact else {
        panic!("a table stays a table");
    };
    assert_eq!(artifact.items.len(), kept);
    assert!(serde_json::to_vec(&artifact).unwrap().len() <= MAX_RESULT_BYTES);
    assert_eq!(
        artifact.items[0]["request_id"], "req_0",
        "the newest rows survive"
    );
    let title = artifact.title.unwrap_or_default();
    assert!(title.contains(&format!("{kept} of 2000 rows")), "{title}");
    assert!(title.contains("--before"), "{title}");
    assert!(title.contains("conversation_audit"), "{title}");
}

#[test]
fn an_oversized_card_becomes_bounded_text_on_a_char_boundary() {
    let body = "é".repeat(MAX_RESULT_BYTES);
    let big = CliArtifact::text(TextArtifact::new(body).with_title("Audit"));
    let bounded = bound_artifact(big, "infra logs audit req_1");
    assert!(bounded.truncated);
    let CliArtifact::Text { artifact } = bounded.artifact else {
        panic!("oversized non-table output becomes text");
    };
    assert!(artifact.content.ends_with("..."));
    assert!(artifact.content.len() <= MAX_RESULT_BYTES + 3);
    assert!(artifact.title.unwrap_or_default().contains("truncated"));
}

#[test]
fn the_bound_is_one_mebibyte() {
    assert_eq!(MAX_RESULT_BYTES, 1024 * 1024);
}

#[test]
fn a_result_under_the_bound_is_not_oversize() {
    let small = table(5, "hello");
    assert!(wire_bytes(&small) < MAX_RESULT_BYTES);
    assert_eq!(oversize_bytes(&small), None);
    assert_eq!(row_count(&small), Some(5));
}

#[test]
fn a_result_at_exactly_the_bound_is_sent_and_one_byte_over_is_not() {
    // Why: TextArtifact serializes its content verbatim, so the wire size is
    // the content plus a fixed envelope; the bound is inclusive.
    let envelope = wire_bytes(&CliArtifact::text(TextArtifact::new(String::new())));
    let at_bound = CliArtifact::text(TextArtifact::new("x".repeat(MAX_RESULT_BYTES - envelope)));
    assert_eq!(wire_bytes(&at_bound), MAX_RESULT_BYTES);
    assert_eq!(oversize_bytes(&at_bound), None);
    let over = CliArtifact::text(TextArtifact::new(
        "x".repeat(MAX_RESULT_BYTES - envelope + 1),
    ));
    assert_eq!(oversize_bytes(&over), Some(MAX_RESULT_BYTES + 1));
    assert_eq!(row_count(&over), None);
}

#[test]
fn the_overflow_pointer_names_the_artifact_page_size_and_rows() {
    let stored = StoredOverflow {
        artifact_id: ArtifactId::new("art_overflow_1"),
        bytes: 2_500_000,
        rows: Some(4_000),
        digest_only: false,
    };
    let pointer = overflow_pointer("infra logs request list --limit 5000", &stored);
    assert!(wire_bytes(&pointer) < 4096, "the pointer is small");
    let CliArtifact::Text { artifact } = pointer else {
        panic!("the pointer is text");
    };
    assert!(
        artifact.content.contains("/admin/artifacts/art_overflow_1"),
        "{}",
        artifact.content
    );
    assert!(
        artifact.content.contains("2500000 bytes (4000 rows)"),
        "{}",
        artifact.content
    );
    assert!(
        artifact.content.contains("stored whole"),
        "{}",
        artifact.content
    );
    assert!(
        artifact.content.contains("--before"),
        "{}",
        artifact.content
    );
    assert!(
        artifact
            .title
            .unwrap_or_default()
            .contains("stored as an artifact")
    );
}

#[test]
fn the_overflow_pointer_says_when_only_the_digest_survived() {
    let stored = StoredOverflow {
        artifact_id: ArtifactId::new("art_overflow_2"),
        bytes: 9 * 1024 * 1024,
        rows: None,
        digest_only: true,
    };
    let CliArtifact::Text { artifact } = overflow_pointer("infra logs audit req_1", &stored) else {
        panic!("the pointer is text");
    };
    assert!(
        artifact.content.contains("Only its digest was kept"),
        "{}",
        artifact.content
    );
    assert!(!artifact.content.contains("rows)"), "{}", artifact.content);
}
