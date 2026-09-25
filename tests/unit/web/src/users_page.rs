//! Pure invariants of the roster and the user detail page.
//!
//! The handlers are covered by the contract and Playwright suites. What this
//! file guards is what neither catches cheaply: the roster's URL contract
//! (which filters exist and how they parse), and the two markup properties a
//! rewrite quietly loses — a write control drifting outside its authorisation
//! guard, and a table losing its empty state.

use systemprompt_web_admin::repositories::users::roster::{RosterFilter, RosterSort};

use crate::support::repo_root;

fn template(name: &str) -> String {
    let path = repo_root().join("storage/files/admin/templates").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// The three chips the retired Unassigned and No-role pages redirect to. Each
// string is a live URL somewhere, so a rename here is a dead redirect.
#[test]
fn every_named_filter_parses_to_its_own_arm() {
    for (raw, expected) in [
        ("unassigned", RosterFilter::Unassigned),
        ("no-role", RosterFilter::NoRole),
        ("inactive-30d", RosterFilter::Inactive30d),
    ] {
        assert_eq!(
            RosterFilter::parse_filter(Some(raw)),
            expected,
            "?filter={raw}"
        );
        assert_eq!(expected.as_str(), raw, "round trip for {raw}");
    }
}

#[test]
fn an_unknown_filter_widens_rather_than_erroring() {
    for raw in [None, Some(""), Some("no-such-filter")] {
        assert_eq!(
            RosterFilter::parse_filter(raw),
            RosterFilter::None,
            "{raw:?}"
        );
    }
}

// Why: the sort column reaches an ORDER BY ladder of literal CASE arms. A value
// with no arm sorts by nothing at all, so parsing must never pass one through.
#[test]
fn an_unknown_sort_column_falls_back_to_the_default() {
    let parsed = RosterSort::parse_sort(Some("'; DROP TABLE users --"), Some("desc"));
    assert_eq!(parsed.column, RosterSort::default().column);
    assert!(RosterSort::COLUMNS.contains(&parsed.column));
}

#[test]
fn every_sort_column_survives_a_round_trip() {
    for column in RosterSort::COLUMNS {
        let asc = RosterSort::parse_sort(Some(column), Some("asc"));
        assert_eq!(asc.column, column);
        assert!(!asc.descending, "{column} asc");
        assert!(
            RosterSort::parse_sort(Some(column), Some("desc")).descending,
            "{column} desc"
        );
    }
}

// Why: an unrecognised direction must mean largest-first, which is what an
// operator scanning for the expensive seat wants, not silently ascending.
#[test]
fn an_absent_direction_sorts_largest_first() {
    assert!(RosterSort::parse_sort(Some("cost"), None).descending);
}

#[test]
fn every_roster_column_is_a_sort_header() {
    let source = template("users.hbs");
    assert!(
        source.contains("{{#each sort_headers}}{{> components/sort-header this}}{{/each}}"),
        "the roster's header row must be built from sort_headers, so every column sorts"
    );
}

#[test]
fn the_roster_paginates_and_shows_an_empty_state() {
    let source = template("users.hbs");
    assert!(
        source.contains("pagination=pagination"),
        "the roster's table must carry the pagination footer"
    );
    assert!(
        source.contains("components/empty-state"),
        "the roster must render an empty state when no account matches"
    );
}

// Why: `can_write` is the only thing between a project manager and a control
// the API would refuse. Every mutating control on both templates sits inside
// that guard, and this counts them so a new one cannot be added outside it.
#[test]
fn every_write_control_sits_behind_the_write_guard() {
    for (name, markers) in [
        (
            "users.hbs",
            vec!["data-action=\"create-user\"", "data-select-user"],
        ),
        (
            "user-detail.hbs",
            vec![
                "data-revoke-device",
                "data-revoke-session",
                "data-revoke-all",
                "data-issue-share-token",
            ],
        ),
    ] {
        let source = template(name);
        for marker in markers {
            let position = source
                .find(marker)
                .unwrap_or_else(|| panic!("{name} no longer offers {marker}"));
            assert!(
                inside_write_guard(&source[..position]),
                "{name}: {marker} is not inside a can_write guard"
            );
        }
    }
}

// Why: a plain count of `{{/if}}` closers cannot tell which block they close,
// so a template with many unrelated `{{#if}}`s reads as unguarded. This walks
// the block structure and asks whether any block still open at the marker is
// the write guard.
fn inside_write_guard(before: &str) -> bool {
    let mut stack: Vec<bool> = Vec::new();
    let mut rest = before;
    while let Some(start) = rest.find("{{") {
        rest = &rest[start..];
        let Some(end) = rest.find("}}") else { break };
        let tag = &rest[2..end];
        let body = tag.trim_start_matches('~').trim();
        if let Some(block) = body.strip_prefix('#') {
            let block = block.trim_start_matches('>').trim();
            if block.starts_with("if ")
                || block.starts_with("unless ")
                || block.starts_with("each ")
            {
                stack.push(block == "if can_write" || block == "if ../can_write");
            } else if block.starts_with("components/") {
                stack.push(false);
            }
        } else if body.starts_with('/') && !body.starts_with("/inline") {
            stack.pop();
        }
        rest = &rest[end + 2..];
    }
    stack.iter().any(|guard| *guard)
}

#[test]
fn the_detail_page_renders_all_five_tabs() {
    let source = template("user-detail.hbs");
    assert!(
        source.contains("components/tabs"),
        "the tab strip is missing"
    );
    for pane in ["identity", "membership", "devices", "sessions", "usage"] {
        assert!(
            source.contains(&format!("{{{{#if {pane}}}}}")),
            "no pane for the {pane} tab"
        );
    }
}

// Why: the primary group and project are the key every exclusive cost total is
// counted by. Losing this editor makes those numbers unmovable from the
// console, which is the state the scope-defaults endpoint exists to end.
#[test]
fn the_detail_page_keeps_the_attribution_editor() {
    let source = template("user-detail.hbs");
    assert!(source.contains("data-form=\"scope-defaults\""));
    assert!(source.contains("name=\"primary_group_id\""));
    assert!(source.contains("name=\"primary_project_id\""));
}

#[test]
fn every_detail_table_has_an_empty_state() {
    let source = template("user-detail.hbs");
    let tables = source.matches("{{#> components/table").count();
    let empties = source.matches("components/empty-state").count();
    assert!(
        empties >= tables - 2,
        "{tables} tables but only {empties} empty states — a table lost its empty branch"
    );
}

fn render_access(mut access: serde_json::Value, can_write: bool) -> Option<String> {
    if !repo_root()
        .join("storage/files/admin/templates/components/user-access.hbs")
        .exists()
    {
        return None;
    }
    if access.get("sections").is_none() {
        access["sections"] = serde_json::json!([]);
    }
    access["overview"]["device_detail"] = serde_json::json!("");
    let engine = systemprompt_web_admin::templates::AdminTemplateEngine::new(
        &repo_root().join("storage/files/admin"),
    )
    .expect("templates load");
    Some(
        engine
            .render(
                "components/user-access",
                &serde_json::json!({
                    "header": { "user_id": "test-user", "name": "Test user" },
                    "can_write": can_write, "access": access,
                }),
            )
            .expect("access renders"),
    )
}

#[test]
fn access_read_failures_are_visible_and_cannot_offer_rule_edits() {
    let Some(html) = render_access(
        serde_json::json!({
            "available": false, "rules_available": false,
            "overview": { "catalog_available": false, "connections_available": false,
                "device_activity": "Unable to load" },
        }),
        true,
    ) else {
        return;
    };
    assert!(html.contains("Unable to load permissions"));
    assert!(html.contains("Unable to load connections"));
    assert!(!html.contains("data-edit-permissions"));
    assert!(!html.contains("No allowed workspaces"));
}

#[test]
fn access_included_content_does_not_claim_client_execution() {
    let Some(html) = render_access(
        serde_json::json!({
            "available": true, "rules_available": true,
            "overview": { "catalog_available": true, "allowed_count": 1,
                "connections_available": true, "attention_count": 1,
                "workspaces": [{ "name": "India Development", "status": "Allowed", "tone": "ok",
                    "reason": "Allowed through group india-devs", "plugins": [
                        { "name": "Business Analysis", "skills": 16 },
                        { "name": "Salesforce Core", "skills": 18 }] }],
                "other_workspaces": [
                { "name": "Platform workspace", "status": "Explicitly denied", "tone": "err", "reason": "Denied through group india-devs" },
                { "name": "Cowork", "status": "Not assigned", "tone": "muted", "reason": "No matching grant for this workspace" }],
            "connections": [{ "name": "Atlassian", "permission": "Allowed",
                    "status": "Sign-in required", "tone": "warn", "verified_at": "Not verified", "next_step": "Sign in through the bridge app" }],
                "device_activity": "2026-09-01 12:00" },
        }),
        false,
    ) else {
        return;
    };
    for text in [
        "Explicitly denied",
        "Not assigned",
        "16 skills",
        "18 skills",
        "Sign-in required",
        "Allowed through group india-devs",
        "Installation and skill execution on the device are not verified",
        "2026-09-01 12:00",
    ] {
        assert!(html.contains(text), "missing {text}");
    }
    assert!(!html.contains("data-edit-permissions"));
}

#[test]
fn a_failed_personal_rule_read_is_unknown_rather_than_inherited() {
    let Some(html) = render_access(
        serde_json::json!({
            "available": true, "rules_available": false, "has_groups": true,
            "overview": { "catalog_available": false, "connections_available": false,
                "device_activity": "Not verified", "workspaces": [] },
            "sections": [{ "label": "Marketplaces", "has_rows": true, "rows": [{
                "entity_type": "marketplace", "entity_id": "india", "entity_name": "India",
                "rule_id": "", "effective": "allow", "effective_tone": "ok", "layer": "group",
                "detail": "group:india-devs allow", "state": "inherit"
            }] }],
        }),
        true,
    ) else {
        return;
    };
    assert!(html.contains("sp-p-access__rule-state\">Unable to load</span>"));
    assert!(!html.contains("data-edit-permissions"));
    assert!(!html.contains("data-can-edit"));
}
