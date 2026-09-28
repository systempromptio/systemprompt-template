//! The access-control plane's drift tiles and action counts, from the counts.

use systemprompt_web_admin::repositories::access_control::drift::DriftCounts;
use systemprompt_web_admin::repositories::sync::access_control::drift_kpis;
use systemprompt_web_admin::repositories::sync::plane::ActionCounts;

fn counts() -> DriftCounts {
    DriftCounts {
        missing_in_db: 3,
        entities_missing: 2,
        changed: 4,
        default_changed: 1,
        only_in_db_code: 5,
        only_in_db_dashboard: 6,
        only_in_db_bundle: 7,
        only_in_db_retire: 8,
        only_in_db_console_retire: 9,
        only_in_db_kept: 10,
        ..DriftCounts::default()
    }
}

#[test]
fn kpis_sum_the_counts_each_tile_names() {
    let kpis = drift_kpis(&counts(), 11);
    let values: Vec<(&str, usize, &str)> =
        kpis.iter().map(|k| (k.label, k.value, k.tone)).collect();
    assert_eq!(
        values,
        vec![
            ("Added in code", 5, "ok"),
            ("Changed in code", 5, "warn"),
            ("Removed from code", 5, "err"),
            ("Written in console", 6, "info"),
            ("Awaiting a bundle", 18, "muted"),
        ]
    );
}

#[test]
fn kpis_are_zero_for_a_clean_plane() {
    let kpis = drift_kpis(&DriftCounts::default(), 0);
    assert_eq!(kpis.len(), 5);
    assert!(kpis.iter().all(|k| k.value == 0));
}

#[test]
fn action_counts_map_each_drift_class_to_its_apply_step() {
    let actions = ActionCounts::from(&counts());
    assert_eq!(actions.insert, 3);
    assert_eq!(actions.insert_entities, 2);
    assert_eq!(actions.update, 5);
    assert_eq!(actions.delete, 8);
    assert_eq!(actions.delete_console, 9);
    assert_eq!(actions.kept, 10);
}
