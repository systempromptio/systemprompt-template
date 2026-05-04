pub(super) use crate::repositories::analytics_grp::dashboard_report::{
    fetch_content_and_breakdown_data, fetch_funnel_and_sparklines,
};

use super::data::{SparkSessionRow, SparkSignupRow};

pub(super) fn build_sparkline_arrays(
    today: chrono::NaiveDate,
    spark_sessions: &[SparkSessionRow],
    spark_signups: &[SparkSignupRow],
) -> (Vec<i64>, Vec<i64>, Vec<String>) {
    let mut spark_sess_arr = Vec::new();
    let mut spark_signup_arr = Vec::new();
    let mut spark_labels = Vec::new();
    for i in (0..7).rev() {
        let day = today - chrono::Duration::days(i);
        spark_labels.push(day.format("%b %d").to_string());
        spark_sess_arr.push(
            spark_sessions
                .iter()
                .find(|r| r.day == day)
                .map_or(0, |r| r.sessions),
        );
        spark_signup_arr.push(
            spark_signups
                .iter()
                .find(|r| r.day == day)
                .map_or(0, |r| r.signups),
        );
    }
    (spark_sess_arr, spark_signup_arr, spark_labels)
}
