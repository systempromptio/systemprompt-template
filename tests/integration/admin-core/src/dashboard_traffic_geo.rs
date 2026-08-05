//! `repositories::dashboard::traffic::queries::list_traffic_country_timeseries`
//! — sessions bucketed by time and country.
//!
//! The query takes the ten busiest countries in the window and relabels every
//! other session `Other`, so the chart's legend stays bounded no matter how
//! long the tail is. That cut, and the `Unknown` label a blank country gets,
//! are the two behaviours worth pinning.

use systemprompt_web_admin::repositories::dashboard::traffic::list_traffic_country_timeseries;

use crate::dashboard_traffic_queries::{SessionSpec, insert_traffic_session};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_traffic_country_timeseries_is_empty_with_no_sessions() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let buckets = list_traffic_country_timeseries(&db.pool, "24 hours", "hour")
        .await
        .expect("country timeseries");

    assert!(buckets.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_traffic_country_timeseries_labels_a_blank_country_unknown() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            country: "",
            ..SessionSpec::default()
        },
    )
    .await;

    let buckets = list_traffic_country_timeseries(&db.pool, "24 hours", "hour")
        .await
        .expect("country timeseries");

    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].country, "Unknown");
    assert_eq!(buckets[0].sessions, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traffic_country_timeseries_folds_everything_past_the_top_ten_into_other() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // Ten countries with two sessions each, plus one with a single session:
    // the eleventh loses the top-10 cut and is relabelled.
    for country in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"] {
        for _ in 0..2 {
            insert_traffic_session(
                &db.pool,
                &SessionSpec {
                    country,
                    ..SessionSpec::default()
                },
            )
            .await;
        }
    }
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            country: "K",
            ..SessionSpec::default()
        },
    )
    .await;

    let buckets = list_traffic_country_timeseries(&db.pool, "24 hours", "hour")
        .await
        .expect("country timeseries");

    let other: i64 = buckets
        .iter()
        .filter(|b| b.country == "Other")
        .map(|b| b.sessions)
        .sum();
    assert_eq!(other, 1);
    assert!(!buckets.iter().any(|b| b.country == "K"));
    db.cleanup().await;
}
