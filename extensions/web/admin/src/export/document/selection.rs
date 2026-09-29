//! Which conversations a JSON Lines export covers: a ticked selection, every
//! context of one session, or the set a list page is showing — resolved
//! through that page's own query type and window kind, so the file answers
//! the question the page was asked.

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId};

use crate::error::{AdminError, AdminResult};
use crate::export::model::{ExportContext, ExportWindow, Window};
use crate::export::window;
use crate::handlers::ssr::ssr_history::{ExportRequest, HistoryView, export_rows};
use crate::repositories::analytics::session_detail::list_session_contexts;
use crate::types::UserContext;
use crate::util::time_range::{TimeRange, TimeRangePreset};

// Why: a bundle carries every body of a conversation; five hundred of them is
// already a file in the tens of megabytes, and the page's filters narrow the
// set further before anyone reaches that.
pub(crate) const MAX_CONVERSATIONS: i64 = 500;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SelectionQuery {
    source: Option<String>,
    ids: Option<String>,
    #[serde(rename = "session_id")]
    session: Option<String>,
}

pub(crate) struct Selection {
    pub context_ids: Vec<ContextId>,
    pub total: i64,
    pub window: Option<ExportWindow>,
}

impl Selection {
    pub(crate) fn capped(&self) -> bool {
        self.total > i64::try_from(self.context_ids.len()).unwrap_or(i64::MAX)
    }
}

// Why: the list page a set comes from — its query type is the filter
// contract and its window kind is the window contract, so the file is the
// page's own answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranscriptSource {
    Sessions,
    Analysis,
    Conversations,
    History,
}

impl TranscriptSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Sessions => "sessions",
            Self::Analysis => "analysis",
            Self::Conversations => "conversations",
            Self::History => "history",
        }
    }

    // Why: this instance's history listing (transcripts and gateway
    // conversations in one union) carries its own window in its query
    // (`days`/`start`/`end`, read by `HistoryWindow`), so the export layer
    // resolves none for it; the ledger pages keep the live one.
    pub(crate) const fn window(self) -> Window {
        match self {
            Self::Sessions | Self::Analysis => Window::Live,
            Self::Conversations | Self::History => Window::None,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        [
            Self::Sessions,
            Self::Analysis,
            Self::Conversations,
            Self::History,
        ]
        .into_iter()
        .find(|s| s.as_str() == value)
    }
}

pub(crate) fn source_of(ctx: &ExportContext<'_>) -> AdminResult<TranscriptSource> {
    let query: SelectionQuery = ctx.query()?;
    let raw = query.source.as_deref().unwrap_or("sessions");
    TranscriptSource::parse(raw).ok_or_else(|| {
        AdminError::BadRequest(format!(
            "Unknown conversation source `{raw}`; use sessions, analysis, conversations or history"
        ))
    })
}

pub(crate) async fn resolve_selection(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
) -> AdminResult<Selection> {
    let query: SelectionQuery = ctx.query()?;
    if let Some(ids) = query.ids.as_deref().filter(|s| !s.trim().is_empty()) {
        return Ok(ticked(ids));
    }
    if let Some(session) = query.session.as_deref().filter(|s| !s.trim().is_empty()) {
        return session_contexts(pool, session).await;
    }
    let source = source_of(ctx)?;
    let w = window::resolve(source.window(), &ctx.query()?)?;
    let mut selection = match source {
        TranscriptSource::Sessions => sessions_set(pool, user, ctx, range_of(required(w)?)).await?,
        TranscriptSource::Analysis => analysis_set(pool, user, ctx, range_of(required(w)?)).await?,
        TranscriptSource::Conversations => history_set(pool, user, ctx, HistoryView::Org).await?,
        TranscriptSource::History => history_set(pool, user, ctx, HistoryView::Own).await?,
    };
    selection.window = w;
    Ok(selection)
}

fn required(w: Option<ExportWindow>) -> AdminResult<ExportWindow> {
    w.ok_or_else(|| AdminError::BadRequest("This export needs a window".to_owned()))
}

// Why: the total counts the ids that name a conversation at all, so a
// malformed id never makes a short selection look capped.
fn ticked(ids: &str) -> Selection {
    let valid: Vec<ContextId> = ids
        .split(',')
        .map(str::trim)
        .filter_map(|s| ContextId::try_new(s).ok())
        .collect();
    let total = i64::try_from(valid.len()).unwrap_or(i64::MAX);
    Selection {
        context_ids: valid
            .into_iter()
            .take(usize::try_from(MAX_CONVERSATIONS).unwrap_or(usize::MAX))
            .collect(),
        total,
        window: None,
    }
}

async fn session_contexts(pool: &PgPool, session: &str) -> AdminResult<Selection> {
    let session_id = SessionId::new(session.trim());
    let rows = list_session_contexts(pool, &session_id).await?;
    let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    Ok(Selection {
        context_ids: rows
            .into_iter()
            .map(|r| r.context_id)
            .take(usize::try_from(MAX_CONVERSATIONS).unwrap_or(usize::MAX))
            .collect(),
        total,
        window: None,
    })
}

const fn range_of(w: ExportWindow) -> TimeRange {
    TimeRange {
        from: w.from,
        to: w.to,
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

// Why: a Stop-hook transcript row has no gateway context, so it has no
// bundle to export and is left out of the set (its count stays in `total`).
async fn history_set(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
    view: HistoryView,
) -> AdminResult<Selection> {
    let (rows, total) = export_rows(
        pool,
        user,
        ExportRequest {
            query: ctx.query()?,
            view,
            limit: MAX_CONVERSATIONS,
        },
    )
    .await?;
    Ok(Selection {
        context_ids: rows.into_iter().filter_map(|r| r.context_id).collect(),
        total,
        window: None,
    })
}

async fn sessions_set(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
    range: TimeRange,
) -> AdminResult<Selection> {
    let (rows, total) = crate::handlers::ssr::ssr_sessions_list::export::export_rows(
        pool,
        user,
        ctx.query()?,
        range,
        MAX_CONVERSATIONS,
    )
    .await?;
    Ok(Selection {
        context_ids: rows.into_iter().map(|r| r.context_id).collect(),
        total,
        window: None,
    })
}

async fn analysis_set(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
    range: TimeRange,
) -> AdminResult<Selection> {
    let result = crate::handlers::ssr::analysis::conversations::export::export_rows(
        pool,
        user,
        ctx.query()?,
        range,
        MAX_CONVERSATIONS,
    )
    .await?;
    Ok(Selection {
        total: result.totals.conversations,
        context_ids: result.rows.into_iter().map(|r| r.context_id).collect(),
        window: None,
    })
}
