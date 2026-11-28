//! Relative times shown beside notices.

use chrono::{DateTime, Duration, Utc};
use systemprompt_harness_views::notifications::strings::relative;

use crate::TestResult;

fn now() -> TestResult<DateTime<Utc>> {
    DateTime::from_timestamp(1_800_000_000, 0).ok_or_else(|| "no such time".into())
}

#[test]
fn a_recent_time_reads_as_just_now_then_minutes_hours_and_days() -> TestResult {
    let now = now()?;
    let ago = |duration: Duration| relative(now - duration, now);
    let just_now = ago(Duration::seconds(30));
    assert!(!just_now.chars().any(|c| c.is_ascii_digit()));
    assert!(ago(Duration::minutes(5)).starts_with("5 min"));
    assert!(ago(Duration::hours(3)).starts_with("3 h"));
    assert!(ago(Duration::days(6)).starts_with("6 d"));
    Ok(())
}

#[test]
fn a_time_a_week_or_more_ago_reads_as_a_date_with_its_year() -> TestResult {
    let now = now()?;
    let shown = relative(now - Duration::days(8), now);
    assert!(shown.contains("2027"), "{shown}");
    Ok(())
}
