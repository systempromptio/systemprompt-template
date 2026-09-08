use chrono::{DateTime, TimeZone, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::governance::warnings::{
    WarningGroupBy, WarningRollupRow, group_warning_rollup,
};

fn at(minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 4, 10, minute, 0).unwrap()
}

fn row(policy: &str, tool: &str, user: &str, count: i64, last: u32) -> WarningRollupRow {
    WarningRollupRow {
        policy: policy.to_owned(),
        tool_name: tool.to_owned(),
        user_id: UserId::new(user.to_owned()),
        count,
        first_seen: at(0),
        last_seen: at(last),
        example_reason: format!("{policy} would have denied {tool} at :{last}"),
    }
}

fn fixture() -> Vec<WarningRollupRow> {
    vec![
        row("secret_scan", "Bash", "ada@example.com", 7, 10),
        row("secret_scan", "Bash", "grace@example.com", 3, 40),
        row("secret_scan", "Read", "ada@example.com", 2, 20),
        row("rate_limit", "Bash", "ada@example.com", 5, 30),
    ]
}

#[test]
fn grouping_by_policy_sums_the_counts() {
    let groups = group_warning_rollup(&fixture(), WarningGroupBy::Policy);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].group, "secret_scan");
    assert_eq!(groups[0].warnings, 12);
    assert_eq!(groups[1].group, "rate_limit");
    assert_eq!(groups[1].warnings, 5);
}

// Why: this is the whole reason the counts are accumulated into sets rather
// than summed. secret_scan appears on three rows but only two distinct tools
// and two distinct users; summing rows would report three of each.
#[test]
fn distinct_tools_and_users_are_counted_not_summed() {
    let groups = group_warning_rollup(&fixture(), WarningGroupBy::Policy);
    let secret_scan = &groups[0];
    assert_eq!(secret_scan.tools, 2);
    assert_eq!(secret_scan.users, 2);
}

#[test]
fn grouping_by_tool_and_by_user_partition_the_same_total() {
    let rows = fixture();
    let total: i64 = rows.iter().map(|r| r.count).sum();
    for by in [
        WarningGroupBy::Policy,
        WarningGroupBy::Tool,
        WarningGroupBy::User,
    ] {
        let summed: i64 = group_warning_rollup(&rows, by)
            .iter()
            .map(|g| g.warnings)
            .sum();
        assert_eq!(
            summed,
            total,
            "grouping by {} lost or duplicated warnings",
            by.as_str()
        );
    }
}

#[test]
fn the_example_reason_comes_from_the_most_recent_row_in_the_group() {
    let groups = group_warning_rollup(&fixture(), WarningGroupBy::Policy);
    let secret_scan = &groups[0];
    assert_eq!(secret_scan.last_seen, Some(at(40)));
    assert_eq!(secret_scan.first_seen, Some(at(0)));
    assert!(
        secret_scan.example_reason.ends_with(":40"),
        "expected the reason from the latest row, got {}",
        secret_scan.example_reason
    );
}

#[test]
fn groups_are_ordered_by_count_then_name() {
    let rows = vec![
        row("b_policy", "Bash", "u1", 4, 10),
        row("a_policy", "Bash", "u1", 4, 10),
        row("c_policy", "Bash", "u1", 9, 10),
    ];
    let names: Vec<String> = group_warning_rollup(&rows, WarningGroupBy::Policy)
        .into_iter()
        .map(|g| g.group)
        .collect();
    assert_eq!(names, ["c_policy", "a_policy", "b_policy"]);
}

#[test]
fn an_empty_window_produces_no_groups() {
    assert!(group_warning_rollup(&[], WarningGroupBy::Policy).is_empty());
}

#[test]
fn an_unknown_group_by_value_falls_back_to_policy() {
    assert_eq!(WarningGroupBy::parse_group_by(None), WarningGroupBy::Policy);
    assert_eq!(
        WarningGroupBy::parse_group_by(Some("nonsense")),
        WarningGroupBy::Policy
    );
    assert_eq!(
        WarningGroupBy::parse_group_by(Some("tool")),
        WarningGroupBy::Tool
    );
    assert_eq!(
        WarningGroupBy::parse_group_by(Some("user")),
        WarningGroupBy::User
    );
}
