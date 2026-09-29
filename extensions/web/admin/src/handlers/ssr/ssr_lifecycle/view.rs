//! Rows for the lifecycle page, shaped from the retention ledger.

use serde::Serialize;

use crate::handlers::ssr::ssr_tools::rows::format_bytes;
use crate::repositories::lifecycle::{ArchiveRow, HealthReportDoc, HealthReportRow, MeasureRow};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MeasureView {
    pub table_name: String,
    pub measured_at: String,
    pub live_rows: i64,
    pub dead_rows: i64,
    pub total: String,
    pub indexes: String,
    pub oldest_row: Option<String>,
    pub window: String,
    pub growth_label: String,
    pub growth_tone: &'static str,
    pub over_window: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ArchiveFileView {
    pub table_name: String,
    pub rows: i64,
    pub size: String,
    pub sha256_short: String,
    pub sha256: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ArchivePeriodView {
    pub tier: String,
    pub period: String,
    pub window: String,
    pub written_at: String,
    pub manifest_href: String,
    pub files: Vec<ArchiveFileView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LifecycleFindingView {
    pub rank: String,
    pub tone: &'static str,
    pub check: String,
    pub detail: String,
}

pub(crate) struct LifecycleView {
    pub measures: Vec<MeasureView>,
    pub periods: Vec<ArchivePeriodView>,
    pub health_run_at: Option<String>,
    pub findings: Vec<LifecycleFindingView>,
    pub counts: (i32, i32, i32),
}

const GROWTH_WARN_PERCENT: i64 = 25;

pub(crate) fn build_view(
    measures: &[MeasureRow],
    archives: &[ArchiveRow],
    health: Option<&HealthReportRow>,
) -> LifecycleView {
    let mut rows: Vec<MeasureView> = measures.iter().map(measure_view).collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.live_rows));
    let (findings, counts, health_run_at) = health.map_or((Vec::new(), (0, 0, 0), None), |h| {
        (
            findings_from(&h.report.0),
            (h.findings_p1, h.findings_p2, h.findings_p3),
            Some(h.run_at.to_rfc3339()),
        )
    });
    LifecycleView {
        measures: rows,
        periods: periods_from(archives),
        health_run_at,
        findings,
        counts,
    }
}

fn measure_view(m: &MeasureRow) -> MeasureView {
    let (growth_label, growth_tone) = match m.bytes_week_ago {
        Some(before) if before > 0 => {
            let pct = (m.total_bytes - before) * 100 / before;
            let tone = if pct > GROWTH_WARN_PERCENT {
                "warn"
            } else {
                "neutral"
            };
            (format!("{pct:+} % / 7 d"), tone)
        },
        _ => ("no baseline yet".to_owned(), "neutral"),
    };
    let over_window = match (m.window_days, m.oldest_row) {
        (Some(days), Some(oldest)) => {
            oldest < m.run_at - chrono::Duration::days(i64::from(days) + 7)
        },
        _ => false,
    };
    MeasureView {
        table_name: m.table_name.clone(),
        measured_at: m.run_at.to_rfc3339(),
        live_rows: m.live_rows,
        dead_rows: m.dead_rows,
        total: format_bytes(m.total_bytes),
        indexes: format_bytes(m.index_bytes),
        oldest_row: m.oldest_row.map(|t| t.date_naive().to_string()),
        window: m
            .window_days
            .map_or_else(|| "kept".to_owned(), |d| format!("{d} d")),
        growth_label,
        growth_tone,
        over_window,
    }
}

fn periods_from(archives: &[ArchiveRow]) -> Vec<ArchivePeriodView> {
    let mut periods: Vec<ArchivePeriodView> = Vec::new();
    for a in archives {
        let href = format!(
            "/admin/lifecycle/archive/{}/{}/{}.jsonl.gz",
            a.tier, a.period, a.table_name
        );
        let file = ArchiveFileView {
            table_name: a.table_name.clone(),
            rows: a.row_count,
            size: format_bytes(a.byte_count),
            sha256_short: a.sha256.chars().take(12).collect(),
            sha256: a.sha256.clone(),
            href,
        };
        match periods
            .iter_mut()
            .find(|p| p.tier == a.tier && p.period == a.period)
        {
            Some(period) => period.files.push(file),
            None => periods.push(ArchivePeriodView {
                tier: a.tier.clone(),
                period: a.period.clone(),
                window: format!(
                    "{} → {}",
                    a.window_from.date_naive(),
                    a.window_to.date_naive()
                ),
                written_at: a.created_at.to_rfc3339(),
                manifest_href: format!(
                    "/admin/lifecycle/archive/{}/{}/manifest.json",
                    a.tier, a.period
                ),
                files: vec![file],
            }),
        }
    }
    periods
}

fn findings_from(report: &HealthReportDoc) -> Vec<LifecycleFindingView> {
    report
        .findings
        .iter()
        .map(|f| LifecycleFindingView {
            tone: match f.rank.as_str() {
                "P1" => "err",
                "P2" => "warn",
                _ => "neutral",
            },
            rank: f.rank.clone(),
            check: f.check.clone(),
            detail: f.detail.clone(),
        })
        .collect()
}

pub(crate) fn valid_tier(tier: &str) -> bool {
    tier == "weekly" || tier == "monthly"
}

pub(crate) fn valid_period(period: &str) -> bool {
    let b = period.as_bytes();
    let year = b.len() >= 5 && b[..4].iter().all(u8::is_ascii_digit) && b[4] == b'-';
    // Why: `2026-W38` and `2026-09` differ only by the week marker; both end
    // in two digits, which is all that is being checked here.
    year && match &b[5..] {
        [b'W', a, c] | [a, c] => a.is_ascii_digit() && c.is_ascii_digit(),
        _ => false,
    }
}

pub(crate) fn valid_file(file: &str) -> bool {
    if file == "manifest.json" {
        return true;
    }
    file.strip_suffix(".jsonl.gz").is_some_and(|table| {
        !table.is_empty() && table.bytes().all(|c| c.is_ascii_lowercase() || c == b'_')
    })
}
