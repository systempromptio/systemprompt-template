use super::types::{LandingPageItem, MetricTuple, SparklineData, TopContentItem};
use super::views::{BarDataView, LandingPageView, MetricRowView, SparklineStrings, TopContentView};

pub(super) fn build_metric_rows(metrics: &[MetricTuple]) -> Vec<MetricRowView> {
    metrics.iter().map(build_metric_row).collect()
}

fn build_metric_row(
    &(label, today_val, yesterday_val, avg_7d, avg_14d, avg_30d, positive_when_up): &MetricTuple,
) -> MetricRowView {
    let is_time_metric = label.contains("Avg Time");

    let value_display = if is_time_metric {
        format_time_ms(today_val)
    } else if (today_val - today_val.floor()).abs() < f64::EPSILON && today_val.abs() < 1_000_000.0
    {
        format!("{today_val:.0}")
    } else {
        format!("{today_val:.1}")
    };

    let fmt_delta = |baseline: f64| -> (String, String, String) {
        if baseline.abs() < f64::EPSILON {
            if today_val.abs() < f64::EPSILON {
                (
                    "--".to_string(),
                    "\u{2014}".to_string(),
                    "neutral".to_string(),
                )
            } else {
                (
                    "+\u{221E}".to_string(),
                    "\u{25B2}".to_string(),
                    if positive_when_up {
                        "positive"
                    } else {
                        "negative"
                    }
                    .to_string(),
                )
            }
        } else {
            let pct = ((today_val - baseline) / baseline) * 100.0;
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
            let sentiment = match (dir, positive_when_up) {
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
    };

    let (yd, ya, ys) = fmt_delta(yesterday_val);
    let (wd, wa, ws) = fmt_delta(avg_7d);
    let (fd, fa, fs) = fmt_delta(avg_14d);
    let (gd, ga, gs) = fmt_delta(avg_30d);

    MetricRowView {
        label: label.to_string(),
        value: value_display,
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

pub(super) fn build_top_content_views(items: &[TopContentItem]) -> Vec<TopContentView> {
    items
        .iter()
        .map(|item| {
            let time_display = format_time_seconds(item.avg_time_seconds);
            let (trend_icon, trend_class) = match item.trend.as_str() {
                "up" => ("\u{25B2}", "positive"),
                "down" => ("\u{25BC}", "negative"),
                _ => ("\u{2014}", "neutral"),
            };
            TopContentView {
                title: item.title.clone(),
                views_7d: item.views_7d,
                views_30d: item.views_30d,
                unique_visitors: item.unique_visitors,
                avg_time: time_display,
                trend_icon,
                trend_class,
                search_impressions: item.search_impressions,
                search_clicks: item.search_clicks,
            }
        })
        .collect()
}

pub(super) fn build_bar_data_from_items(items: &[(&str, i64)]) -> Vec<BarDataView> {
    let total: f64 = i64_to_f64(items.iter().map(|(_, s)| *s).sum::<i64>());
    items
        .iter()
        .map(|(label, sessions)| {
            let pct = if total > 0.0 {
                i64_to_f64(*sessions) / total * 100.0
            } else {
                0.0
            };
            BarDataView {
                label: (*label).to_string(),
                sessions: *sessions,
                pct: format!("{pct:.0}"),
            }
        })
        .collect()
}

pub(super) fn build_landing_page_views(items: &[LandingPageItem]) -> Vec<LandingPageView> {
    items
        .iter()
        .map(|item| LandingPageView {
            page_url: item.page_url.clone(),
            sessions: item.sessions,
            avg_time: format_time_seconds(item.avg_time_seconds),
        })
        .collect()
}

pub(super) fn build_sparkline_strings_from_data(sparklines: &SparklineData) -> SparklineStrings {
    let join_i64 = |arr: &[i64]| -> String {
        arr.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };

    let avg_time = sparklines
        .avg_time_ms
        .iter()
        .map(|v| format!("{:.0}", v / 1000.0))
        .collect::<Vec<_>>()
        .join(",");

    SparklineStrings {
        sessions: join_i64(&sparklines.sessions),
        page_views: join_i64(&sparklines.page_views),
        signups: join_i64(&sparklines.signups),
        avg_time,
    }
}

pub(super) fn i64_to_f64(val: i64) -> f64 {
    val.to_string().parse::<f64>().unwrap_or(0.0)
}

fn format_time_seconds(secs: f64) -> String {
    let s = format!("{:.0}", secs.round()).parse::<i64>().unwrap_or(0);
    if s >= 60 {
        format!("{}m {}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

fn format_time_ms(ms: f64) -> String {
    format_time_seconds(ms / 1000.0)
}
