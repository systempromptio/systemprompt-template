use std::sync::Arc;

use crate::repositories::admin_traffic_reports;
use crate::types::UserContext;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Extension,
};
use sqlx::PgPool;

mod data;
mod metrics;
mod queries;
mod types;
mod views;

use data::{
    build_seo_metrics, build_traffic_overview, build_user_acquisition, fetch_overview_data,
};
use metrics::{
    build_bar_data_from_items, build_landing_page_views, build_metric_rows,
    build_sparkline_strings_from_data, build_top_content_views,
};
use queries::{
    build_sparkline_arrays, fetch_content_and_breakdown_data, fetch_funnel_and_sparklines,
};
use types::{
    ContentFunnel, DeviceBreakdownItem, GeoBreakdownItem, InlineReportData, LandingPageItem,
    SourceBreakdownItem, SparklineData, TopContentItem,
};
use views::{
    ApiErrorResponse, ApiStatusResponse, BreakdownVisibility, ContentVisibility,
    DashboardReportView,
};

pub(super) fn build_dashboard_report(
    report_row: &admin_traffic_reports::AdminTrafficReportRow,
) -> serde_json::Value {
    let report: InlineReportData = match serde_json::from_value(report_row.report_data.clone()) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to deserialize report data");
            return serde_json::Value::Object(serde_json::Map::new());
        }
    };

    let traffic_overview = build_metric_rows(&report.traffic_overview.metric_values());
    let user_acquisition = build_metric_rows(&report.user_acquisition.metric_values());
    let top_content = build_top_content_views(&report.top_content);
    let geo_breakdown = build_bar_data_from_items(
        &report
            .geo_breakdown
            .iter()
            .map(|g| (g.country.as_str(), g.sessions))
            .collect::<Vec<_>>(),
    );
    let device_breakdown = build_bar_data_from_items(
        &report
            .device_breakdown
            .iter()
            .map(|d| (d.device.as_str(), d.sessions))
            .collect::<Vec<_>>(),
    );
    let source_breakdown = build_bar_data_from_items(
        &report
            .source_breakdown
            .iter()
            .map(|s| (s.source.as_str(), s.sessions))
            .collect::<Vec<_>>(),
    );
    let landing_pages = build_landing_page_views(&report.top_landing_pages);
    let sparklines = build_sparkline_strings_from_data(&report.sparklines);

    let report_date = report_row.report_date.format("%B %d, %Y").to_string();
    let generated_at = report_row.generated_at.format("%H:%M UTC").to_string();

    let content_visibility = ContentVisibility {
        has_top_content: !top_content.is_empty(),
        has_landing_pages: !landing_pages.is_empty(),
    };
    let breakdown_visibility = BreakdownVisibility {
        has_geo: !geo_breakdown.is_empty(),
        has_devices: !device_breakdown.is_empty(),
        has_sources: !source_breakdown.is_empty(),
    };

    let view = DashboardReportView {
        report_date,
        generated_at,
        traffic_overview,
        user_acquisition,
        top_content,
        seo_impressions: report.seo_metrics.total_impressions,
        seo_clicks: report.seo_metrics.total_clicks,
        seo_ctr: format!("{:.1}%", report.seo_metrics.avg_ctr),
        seo_indexed: report.seo_metrics.total_indexed_pages,
        seo_avg_position: format!("{:.1}", report.seo_metrics.avg_search_position),
        geo_breakdown,
        device_breakdown,
        source_breakdown,
        content_funnel: report.content_funnel,
        landing_pages,
        sparkline_sessions: sparklines.sessions,
        sparkline_page_views: sparklines.page_views,
        sparkline_signups: sparklines.signups,
        sparkline_avg_time: sparklines.avg_time,
        content_visibility,
        breakdown_visibility,
    };

    serde_json::to_value(&view).unwrap_or_else(|_| serde_json::Value::Null)
}

pub async fn handle_generate_traffic_report(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiErrorResponse {
                error: "admin_required".to_string(),
            }),
        )
            .into_response();
    }

    match generate_report_inline(&pool).await {
        Ok(()) => Json(ApiStatusResponse { status: "ok" }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate admin traffic report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn generate_report_inline(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use chrono::Timelike;

    let today = chrono::Utc::now().date_naive();
    let hour = chrono::Utc::now().hour();
    let period = if hour < 14 { "am" } else { "pm" };

    let report_data = fetch_report_data_inline(pool).await?;
    let report_json = serde_json::to_value(&report_data)?;

    sqlx::query!(
        r#"
        INSERT INTO admin_traffic_reports (report_date, report_period, report_data, generated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (report_date, report_period)
        DO UPDATE SET report_data = $3, generated_at = NOW()
        "#,
        today,
        period,
        &report_json,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_report_data_inline(
    pool: &PgPool,
) -> Result<InlineReportData, Box<dyn std::error::Error + Send + Sync>> {
    let today = chrono::Utc::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);

    let (sessions_row, pv_row, acq_row) = fetch_overview_data(pool, today, yesterday).await?;
    let (top_content, seo, geo, devices, sources) = fetch_content_and_breakdown_data(pool).await?;
    let (funnel, landing, spark_sessions, spark_signups) =
        fetch_funnel_and_sparklines(pool).await?;

    let (spark_sess_arr, spark_signup_arr, spark_labels) =
        build_sparkline_arrays(today, &spark_sessions, &spark_signups);

    Ok(InlineReportData {
        traffic_overview: build_traffic_overview(&sessions_row, &pv_row),
        user_acquisition: build_user_acquisition(&acq_row),
        top_content: top_content
            .into_iter()
            .map(|r| TopContentItem {
                title: r.title,
                slug: r.slug,
                views_7d: r.views_7d,
                views_30d: r.views_30d,
                unique_visitors: r.unique_visitors,
                avg_time_seconds: r.avg_time_seconds,
                trend: r.trend,
                search_impressions: r.search_impressions,
                search_clicks: r.search_clicks,
            })
            .collect(),
        seo_metrics: build_seo_metrics(&seo),
        geo_breakdown: geo
            .into_iter()
            .map(|r| GeoBreakdownItem {
                country: r.country,
                sessions: r.sessions,
            })
            .collect(),
        device_breakdown: devices
            .into_iter()
            .map(|r| DeviceBreakdownItem {
                device: r.device,
                sessions: r.sessions,
            })
            .collect(),
        source_breakdown: sources
            .into_iter()
            .map(|r| SourceBreakdownItem {
                source: r.source,
                sessions: r.sessions,
            })
            .collect(),
        content_funnel: ContentFunnel {
            total_published: funnel.total_published,
            avg_views_per_piece: funnel.avg_views,
            total_shares: funnel.total_shares,
            total_comments: funnel.total_comments,
        },
        sparklines: SparklineData {
            sessions: spark_sess_arr,
            page_views: vec![],
            signups: spark_signup_arr,
            avg_time_ms: vec![],
            labels: spark_labels,
        },
        top_landing_pages: landing
            .into_iter()
            .map(|r| LandingPageItem {
                page_url: r.page_url,
                sessions: r.sessions,
                avg_time_seconds: r.avg_time_seconds,
            })
            .collect(),
    })
}
