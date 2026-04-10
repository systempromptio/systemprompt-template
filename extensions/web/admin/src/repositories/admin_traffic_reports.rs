use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct AdminTrafficReportRow {
    pub id: String,
    pub report_date: chrono::NaiveDate,
    pub report_period: String,
    // JSON: aggregated traffic report from jsonb column
    pub report_data: serde_json::Value,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_latest_report(
    pool: &PgPool,
) -> Result<Option<AdminTrafficReportRow>, sqlx::Error> {
    sqlx::query_as!(
        AdminTrafficReportRow,
        r#"
        SELECT
            id,
            report_date,
            report_period,
            report_data,
            generated_at
        FROM admin_traffic_reports
        ORDER BY report_date DESC, generated_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_report(
    pool: &PgPool,
    report_date: chrono::NaiveDate,
    period: &str,
    data: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO admin_traffic_reports (report_date, report_period, report_data, generated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (report_date, report_period)
        DO UPDATE SET report_data = $3, generated_at = NOW()
        "#,
        report_date,
        period,
        data,
    )
    .execute(pool)
    .await?;
    Ok(())
}
