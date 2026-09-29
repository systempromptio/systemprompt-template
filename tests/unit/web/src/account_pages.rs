//! The account templates — profile, history, connect, connectors, setup — are
//! held to the design system rather than to a screenshot.
//!
//! Each of these pages carried its own header, its own card and its own list
//! markup before the rebuild, which is how the account section came to look
//! like a second product. Two properties keep that from happening again: every
//! account page opens with the shared page header and breadcrumb trail, and
//! none of the retired bespoke class families survives anywhere in the
//! templates. Both are textual facts about the template source, so they hold
//! without a server or a browser.

use crate::support::repo_root;

const PAGES: [&str; 5] = ["profile", "history", "connect", "connectors", "setup"];

// Every class family the account pages used to define for themselves. Their
// stylesheets are deleted, so any surviving reference styles nothing at all.
const RETIRED: [&str; 16] = [
    "sp-connection",
    "sp-p-settings",
    "sp-profile-hero",
    "sp-profile-card",
    "sp-profile-row",
    "sp-quickstat",
    "sp-tile",
    "sp-modellist",
    "sp-agentlist",
    "sp-conv-table",
    "sp-deflist",
    "sp-settings-form-grid",
    "sp-settings-field",
    "sp-danger-zone-item",
    "sp-setup-stepper",
    "sp-setup-phase",
];

fn template(name: &str) -> String {
    let path = repo_root().join(format!("storage/files/admin/templates/{name}.hbs"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("account template {} is tracked: {e}", path.display()))
}

#[test]
fn every_account_page_opens_with_the_shared_header_and_trail() {
    let mut missing = Vec::new();
    for name in PAGES {
        let body = template(name);
        if !body.contains("components/breadcrumbs") {
            missing.push(format!("{name}: no breadcrumb trail"));
        }
        if !body.contains("components/page-header") {
            missing.push(format!("{name}: no shared page header"));
        }
    }
    assert!(
        missing.is_empty(),
        "account page(s) not wearing the shell:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_account_page_keeps_a_retired_bespoke_class() {
    let mut found = Vec::new();
    for name in PAGES {
        let body = template(name);
        for class in RETIRED {
            if body.contains(class) {
                found.push(format!("{name}: {class}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "retired account-page class(es) still referenced — their CSS is deleted, so they style \
         nothing:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_retired_account_stylesheets_are_gone() {
    let css = repo_root().join("storage/files/css/admin");
    let mut left = Vec::new();
    for file in [
        "20-page-profile2.css",
        "20-page-profile3.css",
        "20-page-profile4-connect.css",
        "20-page-profile5-tiles.css",
        "20-page-profile6-sections.css",
        "20-page-profile7-deflist.css",
        "90-setup.css",
        "90-setup-actions.css",
        "05-connection.css",
    ] {
        if css.join(file).exists() {
            left.push(file);
        }
    }
    assert!(
        left.is_empty(),
        "account page stylesheets that the rebuild replaced are still on disk: {left:?}"
    );
}

#[test]
fn every_account_table_carries_a_screen_reader_caption() {
    let mut bare = Vec::new();
    for name in PAGES {
        let body = template(name);
        let tables = body.matches("{{#> components/table").count();
        let captions = body.matches("caption=").count();
        if tables > captions {
            bare.push(format!("{name}: {tables} tables, {captions} captions"));
        }
    }
    assert!(
        bare.is_empty(),
        "account page table(s) with no caption — a caption is the only name a screen reader has \
         for a table:\n{}",
        bare.join("\n")
    );
}

// The connect card is the reason the connect page exists: one issued code,
// three clients, each with its guide and every command the code fills. These
// are the facts a redesign must keep, and they are all textual.
#[test]
fn the_connect_card_is_tabbed_documented_and_quiet() {
    let body = template("connect");
    let mut missing = Vec::new();
    for needle in [
        r#"data-tabs="profile-connect""#,
        r#"data-tab="claude-code""#,
        r#"data-tab="claude-desktop""#,
        r#"data-tab="opencode""#,
        r#"href="/documentation/connect-claude-code""#,
        r#"href="/documentation/bridge-install""#,
        r#"href="/documentation/connect-opencode""#,
        r#"data-connect-field="install_command""#,
        r#"data-connect-field="opencode_install_command""#,
        r#"data-connect-field="desktop_windows_login_command""#,
        r#"href="/files/downloads/systemprompt-bridge-macos.dmg""#,
        "components/sp-tabs.js",
    ] {
        if !body.contains(needle) {
            missing.push(needle);
        }
    }
    assert!(
        missing.is_empty(),
        "connect card lost:\n{}",
        missing.join("\n")
    );
    assert_eq!(
        body.matches(r#"role="tabpanel""#).count(),
        3,
        "one panel per client"
    );
    assert!(
        !body.contains("not available yet"),
        "the macOS bridge ships; the connect page must not say otherwise"
    );
}

// The profile carries the person's usage and nothing else: the connect
// wizard, the connector list and the settings form each have a page of their
// own. This installation keeps settings on `/admin/settings`.
#[test]
fn the_profile_holds_usage_only() {
    let settings = template("settings");
    for needle in [
        r#"id="settings-form""#,
        r#"id="settings-timezone""#,
        r#"id="delete-account-btn""#,
    ] {
        assert!(settings.contains(needle), "settings page lost {needle}");
    }
    let body = template("profile");
    for needle in [r#"href="/admin/connect""#, r#"href="/admin/connectors""#] {
        assert!(body.contains(needle), "profile lost {needle}");
    }
    for stray in [
        "data-connect-field",
        "data-connected-accounts",
        "data-tabs=",
    ] {
        assert!(!body.contains(stray), "profile still carries {stray}");
    }
    let tables = body.matches("{{#> components/table").count();
    assert!(
        tables <= 4,
        "profile carries {tables} tables; the analytics breakdowns belong to History"
    );
}

// The connectors page leads with the summary and puts the test flow inside
// every card, so a person never has to scroll past their usage to find it.
#[test]
fn the_connectors_page_leads_with_summary_and_owns_the_test_flow() {
    let body = template("connectors");
    for needle in [
        r#"data-summary="connected""#,
        r#"data-summary="needs_attention""#,
        r#"data-summary="not_connected""#,
        r#"data-summary="configured""#,
        "data-connector-grid",
        "sp-connector__check",
        "pages/connectors.js",
    ] {
        assert!(body.contains(needle), "connectors page lost {needle}");
    }
}
