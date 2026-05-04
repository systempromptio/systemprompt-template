use sqlx::PgPool;

pub async fn upsert_traffic_report(
    pool: &PgPool,
    report_date: chrono::NaiveDate,
    period: &str,
    report_json: &serde_json::Value,
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
        report_json,
    )
    .execute(pool)
    .await?;
    Ok(())
}
