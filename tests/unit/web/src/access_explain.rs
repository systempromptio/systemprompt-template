//! The "Why?" explainer: every band asked alone through the real resolver,
//! then marked decided, outranked, no rule or not held against the one
//! decision the whole subject gets.

use chrono::Utc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    ParentChainIndex, RuleType, SubjectAttributes, SubjectDimension,
};
use systemprompt_web_admin::repositories::users::access_control::MatrixSubject;
use systemprompt_web_admin::repositories::users::access_control::explain::{
    BandExplanation, ExplainInput, Explanation, explain,
};
use systemprompt_web_admin::types::access_control::{AccessControlRule, AccessDecision};

fn group_band() -> RuleType {
    RuleType::extension("group").expect("group band")
}

fn rule(rule_type: RuleType, value: &str, access: AccessDecision) -> AccessControlRule {
    AccessControlRule {
        id: format!("{rule_type}-{value}"),
        entity_type: "mcp_server".to_owned(),
        entity_id: "atlassian".to_owned(),
        rule_type,
        rule_value: value.to_owned(),
        access,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn subject(roles: &[&str], groups: &[&str]) -> MatrixSubject {
    let mut attributes = SubjectAttributes::new();
    if !groups.is_empty() {
        attributes.insert(
            group_band(),
            groups.iter().map(|g| (*g).to_owned()).collect(),
        );
    }
    MatrixSubject {
        id: UserId::new("someone@example.com"),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        attributes,
    }
}

fn run(subject: &MatrixSubject) -> Explanation {
    let rules = [
        rule(group_band(), "india-devs", AccessDecision::Allow),
        rule(RuleType::ROLE, "developer", AccessDecision::Deny),
    ];
    let dimensions = [SubjectDimension {
        rule_type: group_band(),
        label: "group",
        precedence: 150,
    }];
    explain(&ExplainInput {
        rules: &rules,
        entity_type: "mcp_server",
        entity_id: "atlassian",
        subject,
        dimensions: &dimensions,
        default_open: false,
        chains: &ParentChainIndex::default(),
    })
}

fn band<'a>(explanation: &'a Explanation, name: &str) -> &'a BandExplanation {
    explanation
        .bands
        .iter()
        .find(|b| b.band == name)
        .unwrap_or_else(|| panic!("no {name} band: {explanation:?}"))
}

#[test]
fn the_narrower_band_decides_and_the_wider_one_is_outranked() {
    let explained = run(&subject(&["developer"], &["india-devs"]));
    assert_eq!(explained.effective, "allow", "{explained:?}");
    assert_eq!(explained.layer, "group");
    assert_eq!(band(&explained, "group").verdict, "decided");
    assert_eq!(band(&explained, "group").outcome, "allow");
    assert_eq!(band(&explained, "role").verdict, "outranked");
    assert_eq!(band(&explained, "role").outcome, "deny");
    assert_eq!(band(&explained, "user").verdict, "no_rule");
}

#[test]
fn a_band_the_person_holds_nothing_in_is_not_held() {
    let explained = run(&subject(&["developer"], &[]));
    assert_eq!(explained.effective, "deny", "{explained:?}");
    assert_eq!(explained.layer, "role");
    assert_eq!(band(&explained, "group").verdict, "not_held");
    assert_eq!(band(&explained, "role").verdict, "decided");
}

#[test]
fn bands_are_listed_narrowest_first() {
    let explained = run(&subject(&[], &[]));
    let order: Vec<u16> = explained.bands.iter().map(|b| b.precedence).collect();
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(order, sorted);
    assert_eq!(
        explained.bands.first().map(|b| b.band.as_str()),
        Some("user")
    );
}
