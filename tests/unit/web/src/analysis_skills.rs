//! The pure halves of the Analysis skill pages: window-bound parsing and
//! rendering (shared with the export dialog) and the `plugin:skill` key
//! parser every skill URL goes through.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, TimeZone, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::test_support::{
    MarketplaceAudience, parse_skill_key, parse_window_bound, render_window_bound,
};

fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, 0)
        .single()
        .unwrap_or_default()
}

#[test]
fn window_bounds_accept_dates_datetimes_and_rfc3339() {
    assert_eq!(parse_window_bound("2026-09-01"), Some(at(2026, 9, 1, 0, 0)));
    assert_eq!(
        parse_window_bound("2026-09-01T10:30"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(
        parse_window_bound("2026-09-01T10:30:00"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(
        parse_window_bound("2026-09-01T12:30:00+02:00"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(parse_window_bound("yesterday"), None);
    assert_eq!(parse_window_bound("2026-13-01"), None);
}

#[test]
fn window_bounds_render_as_dates_on_midnight_and_datetimes_otherwise() {
    assert_eq!(render_window_bound(at(2026, 9, 1, 0, 0)), "2026-09-01");
    assert_eq!(
        render_window_bound(at(2026, 9, 1, 10, 30)),
        "2026-09-01T10:30:00"
    );
    let round_trip = parse_window_bound(&render_window_bound(at(2026, 9, 1, 0, 0)));
    assert_eq!(round_trip, Some(at(2026, 9, 1, 0, 0)));
}

#[test]
fn a_skill_key_is_plugin_colon_skill_and_nothing_else() {
    let key = parse_skill_key("astound-india-ba:ba-bug-logging").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(key.plugin_id.as_str(), "astound-india-ba");
    assert_eq!(key.skill, "astound-india-ba:ba-bug-logging");
    assert!(parse_skill_key("who-am-i").is_err(), "no plugin prefix");
    assert!(parse_skill_key(":who-am-i").is_err(), "empty plugin");
    assert!(parse_skill_key("a:").is_err(), "empty skill");
    assert!(parse_skill_key("a:b c").is_err(), "whitespace");
}

fn audience(pairs: &[(&str, &[&str])]) -> MarketplaceAudience {
    MarketplaceAudience {
        users_by_marketplace: pairs
            .iter()
            .map(|(m, users)| {
                (
                    (*m).to_owned(),
                    users
                        .iter()
                        .map(|u| UserId::new(*u))
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<_, _>>(),
    }
}

// Why: installs are receipts from anyone who ever installed; the install
// rate must count only the installed the rules entitle, or one consumer
// outside the entitlement reads as 200%.
#[test]
fn installed_are_counted_against_entitlement_per_marketplace_and_overall() {
    let audience = audience(&[("commons", &["ann"]), ("sales", &["ann", "bob"])]);
    let consumers = ["ann".to_owned(), "zed".to_owned()];
    assert_eq!(audience.entitled_among("commons", &consumers), 1);
    assert_eq!(audience.entitled_among("sales", &consumers), 1);
    assert_eq!(audience.entitled_among("nowhere", &consumers), 0);
    assert_eq!(audience.entitled_among_any(&consumers), 1);
    assert_eq!(audience.users_reaching_marketplace("commons"), 1);
    assert_eq!(audience.users_reaching_any_marketplace(), 2);
}
