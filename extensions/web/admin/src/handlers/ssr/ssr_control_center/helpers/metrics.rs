use crate::numeric;
use crate::repositories::daily_summaries::DailySummaryRow;

use super::super::types::MetricRow;

#[derive(Debug, Clone, Copy)]
pub struct MetricRowInput {
    pub label: &'static str,
    pub today_val: f64,
    pub yesterday_val: Option<f64>,
    pub avg_7d: Option<f64>,
    pub avg_14d: Option<f64>,
    pub global_avg: Option<f64>,
    pub positive_when_up: bool,
}

pub fn make_metric_row(input: &MetricRowInput) -> MetricRow {
    let fmt_delta = |baseline: Option<f64>| -> (String, String, String) {
        match baseline {
            None => (
                "--".to_string(),
                "\u{2014}".to_string(),
                "neutral".to_string(),
            ),
            Some(b) if (b).abs() < f64::EPSILON => {
                if input.today_val.abs() < f64::EPSILON {
                    (
                        "--".to_string(),
                        "\u{2014}".to_string(),
                        "neutral".to_string(),
                    )
                } else {
                    (
                        "+\u{221E}".to_string(),
                        "\u{25B2}".to_string(),
                        if input.positive_when_up {
                            "positive"
                        } else {
                            "negative"
                        }
                        .to_string(),
                    )
                }
            }
            Some(b) => {
                let pct = ((input.today_val - b) / b) * 100.0;
                let arrow = if pct > 1.0 {
                    "\u{25B2}"
                } else if pct < -1.0 {
                    "\u{25BC}"
                } else {
                    "\u{2014}"
                };
                let dir = if pct > 1.0 {
                    "up"
                } else if pct < -1.0 {
                    "down"
                } else {
                    "flat"
                };
                let sentiment = match (dir, input.positive_when_up) {
                    ("up", true) | ("down", false) => "positive",
                    ("down", true) | ("up", false) => "negative",
                    _ => "neutral",
                };
                (
                    format!("{:.1}%", pct.abs()),
                    arrow.to_string(),
                    sentiment.to_string(),
                )
            }
        }
    };

    let (yd, ya, ys) = fmt_delta(input.yesterday_val);
    let (wd, wa, ws) = fmt_delta(input.avg_7d);
    let (fd, fa, fs) = fmt_delta(input.avg_14d);
    let (gd, ga, gs) = fmt_delta(input.global_avg);

    let value = if (input.today_val - input.today_val.floor()).abs() < f64::EPSILON {
        format!("{:.0}", input.today_val)
    } else {
        format!("{:.1}", input.today_val)
    };

    MetricRow {
        label: input.label,
        value,
        yesterday_delta: yd,
        yesterday_arrow: ya,
        yesterday_sentiment: ys,
        week_delta: wd,
        week_arrow: wa,
        week_sentiment: ws,
        fortnight_delta: fd,
        fortnight_arrow: fa,
        fortnight_sentiment: fs,
        global_delta: gd,
        global_arrow: ga,
        global_sentiment: gs,
    }
}

pub fn avg_field(
    days: &[DailySummaryRow],
    f: impl Fn(&DailySummaryRow) -> f64,
) -> Option<f64> {
    if days.is_empty() {
        return None;
    }
    Some(days.iter().map(&f).sum::<f64>() / numeric::usize_to_f64(days.len()))
}

pub fn parse_summary_parts(summary: &str) -> (String, Vec<String>) {
    let mut goal_lines = Vec::new();
    let mut outcomes = Vec::new();

    for line in summary.lines() {
        let trimmed = line.trim();
        if let Some(bullet) = trimmed.strip_prefix("- ") {
            outcomes.push(bullet.to_string());
        } else if !trimmed.is_empty() && outcomes.is_empty() {
            goal_lines.push(trimmed.to_string());
        }
    }

    let goal_summary = goal_lines.join(" ");
    (goal_summary, outcomes)
}
