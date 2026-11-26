//! The session's stored artifacts on the request audit page.
//!
//! Every stored tool result of the same session, which is how an oversize CLI
//! result that overflowed into its own artifact is found from the request
//! that asked for it.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::repositories::analysis::tools::{
    ToolActivityFilter, ToolActivityPage, ToolActivityRow, ToolBreakdownBy, ToolSort,
    load_tool_activity_page,
};

const ARTIFACT_LIMIT: i64 = 50;

#[derive(Debug, Default, Serialize)]
pub(super) struct ArtifactsView {
    pub rows: Vec<ArtifactLinkView>,
    pub has_rows: bool,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ArtifactLinkView {
    pub artifact_key: String,
    pub url: String,
    pub tool_name: String,
    pub server_name: String,
    pub artifact_type: String,
    pub title: String,
    pub bytes: String,
    pub request_id: Option<String>,
    pub is_this_request: bool,
    pub is_error: bool,
    pub is_structured: bool,
    pub created_at: String,
}

pub(super) async fn load_artifacts(
    pool: &PgPool,
    session_id: &SessionId,
    primary_request_id: Option<&str>,
) -> ArtifactsView {
    let filter = ToolActivityFilter {
        session: Some(session_id.as_str().to_owned()),
        artifacts_only: true,
        ..ToolActivityFilter::default()
    };
    let page = ToolActivityPage {
        sort: ToolSort::default(),
        descending: true,
        limit: ARTIFACT_LIMIT,
        offset: 0,
        breakdown: ToolBreakdownBy::default(),
    };
    let rows = match load_tool_activity_page(pool, &filter, page).await {
        Ok(result) => result.rows,
        Err(e) => {
            tracing::warn!(error = %e, "artifact list for the request page failed");
            Vec::new()
        },
    };
    let rows: Vec<ArtifactLinkView> = rows
        .iter()
        .filter_map(|r| artifact_link(r, primary_request_id))
        .collect();
    ArtifactsView {
        count: rows.len(),
        has_rows: !rows.is_empty(),
        rows,
    }
}

fn artifact_link(
    r: &ToolActivityRow,
    primary_request_id: Option<&str>,
) -> Option<ArtifactLinkView> {
    let artifact_key = r.artifact_key.clone()?;
    Some(ArtifactLinkView {
        url: format!("/admin/artifacts/{}", urlencoding::encode(&artifact_key)),
        artifact_key,
        tool_name: r.tool_name.clone().unwrap_or_else(|| "—".to_owned()),
        server_name: r.server_name.clone().unwrap_or_default(),
        artifact_type: r.artifact_type.clone().unwrap_or_default(),
        title: r.artifact_title.clone().unwrap_or_default(),
        bytes: r
            .payload_bytes
            .map_or_else(|| "—".to_owned(), |b| b.to_string()),
        is_this_request: primary_request_id.is_some_and(|id| r.request_id.as_deref() == Some(id)),
        request_id: r.request_id.clone(),
        is_error: r.is_error,
        is_structured: r.is_structured,
        created_at: r
            .occurred_at
            .map(|t| {
                t.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default(),
    })
}
