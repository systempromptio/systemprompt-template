//! View assembly for the Sessions tab.
//!
//! Every figure is the client's own statusline total, not a gateway
//! measurement, and the page labels it so on the strip rather than in a
//! footnote. Ratings come from `session_ratings`, which people fill in by
//! hand, so the rated count is stated beside the mean.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::site::sessions::{SessionCostRow, SessionRatingStats};

use super::context::{KpiTile, SessionCostRowView, SessionsTabView};
use super::tab_models::{per, share};
use super::view::{compact, format_date};
use super::{AnalyticsDashboardQuery, PAGE_SIZE, urls};

pub(super) struct SessionsInput<'a> {
    pub rows: &'a [SessionCostRow],
    pub total_rows: i64,
    pub ratings: SessionRatingStats,
    pub page: i64,
}

pub(super) fn sessions_tab(
    input: &SessionsInput<'_>,
    query: &AnalyticsDashboardQuery,
) -> SessionsTabView {
    let max = input
        .rows
        .iter()
        .map(|r| r.total_cost_microdollars)
        .max()
        .unwrap_or(0);
    let views: Vec<SessionCostRowView> = input.rows.iter().map(|r| row(r, max)).collect();

    let pagination = (input.total_rows > PAGE_SIZE).then(|| {
        urls::build_pagination(
            query,
            PageWindow::new(
                input.page,
                PAGE_SIZE,
                input.total_rows,
                i64::try_from(input.rows.len()).unwrap_or(PAGE_SIZE),
                "sessions",
            ),
        )
    });

    SessionsTabView {
        kpis: kpis(input),
        session_count: input.total_rows,
        has_rows: !views.is_empty(),
        rows: views,
        pagination,
    }
}

fn row(r: &SessionCostRow, max: i64) -> SessionCostRowView {
    SessionCostRowView {
        session_short: short_id(r.session_id.as_str()),
        model_display: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        cost_display: format_cost(r.total_cost_microdollars),
        share_pct: share(r.total_cost_microdollars, max),
        context_display: compact(r.context_window_size),
        input_display: compact(r.input_tokens),
        output_display: compact(r.output_tokens),
        cache_display: compact(r.cache_read_tokens),
        rating_display: r
            .rating
            .map_or_else(|| "—".to_owned(), |v| format!("{v}/5")),
        outcome_display: r
            .outcome
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "—".to_owned()),
        updated_display: format_date(r.updated_at),
        detail_url: format!(
            "/admin/sessions/{}",
            urlencoding::encode(r.session_id.as_str())
        ),
        user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
        session_id: r.session_id.clone(),
        user_id: r.user_id.clone(),
    }
}

// Why: a session id is a uuid nobody reads in full; the head is enough to
// recognise a row, and the cell links to the session itself.
fn short_id(id: &str) -> String {
    id.chars().take(12).collect()
}

fn kpis(input: &SessionsInput<'_>) -> Vec<KpiTile> {
    let cost: i64 = input.rows.iter().map(|r| r.total_cost_microdollars).sum();
    let context: i64 = input
        .rows
        .iter()
        .map(|r| r.context_window_size)
        .max()
        .unwrap_or(0);
    let ratings = input.ratings;
    vec![
        KpiTile {
            label: "Sessions".to_owned(),
            value: input.total_rows.to_string(),
            sub: "with a client-reported cost snapshot".to_owned(),
            tone: "accent",
        },
        KpiTile {
            label: "Cost on this page".to_owned(),
            value: format_cost(cost),
            sub: format!(
                "{} per session · client-reported",
                format_cost(per(cost, i64::try_from(input.rows.len()).unwrap_or(0)))
            ),
            tone: "ok",
        },
        KpiTile {
            label: "Peak context window".to_owned(),
            value: compact(context),
            sub: "largest on this page".to_owned(),
            tone: "warn",
        },
        KpiTile {
            label: "Rated sessions".to_owned(),
            value: ratings.rated.to_string(),
            sub: ratings.avg_rating.map_or_else(
                || "nobody rated a session in this window".to_owned(),
                |v| {
                    format!(
                        "mean {v:.1}/5 · {} good, {} poor",
                        ratings.good, ratings.poor
                    )
                },
            ),
            tone: if ratings.poor > ratings.good {
                "err"
            } else {
                "ok"
            },
        },
    ]
}
