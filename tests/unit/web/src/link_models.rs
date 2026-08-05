//! `UtmParams::to_query_string` emits the five parameters in a fixed order and
//! percent-encodes every value, so a campaign URL is stable and safe to paste
//! into an `href` regardless of what the campaign was named.

use systemprompt_web_shared::models::{DestinationType, LinkType, UtmParams};

fn params() -> UtmParams {
    UtmParams {
        source: None,
        medium: None,
        campaign: None,
        term: None,
        content: None,
    }
}

#[test]
fn all_five_parameters_emit_in_declaration_order() {
    let utm = UtmParams {
        source: Some("linkedin".to_owned()),
        medium: Some("social".to_owned()),
        campaign: Some("launch".to_owned()),
        term: Some("governance".to_owned()),
        content: Some("cta-a".to_owned()),
    };
    assert_eq!(
        utm.to_query_string(),
        "utm_source=linkedin&utm_medium=social&utm_campaign=launch&utm_term=governance&utm_content=cta-a"
    );
}

#[test]
fn absent_parameters_are_skipped_without_leaving_empty_pairs() {
    let utm = UtmParams {
        medium: Some("email".to_owned()),
        ..params()
    };
    assert_eq!(utm.to_query_string(), "utm_medium=email");

    let utm = UtmParams {
        source: Some("x".to_owned()),
        content: Some("footer".to_owned()),
        ..params()
    };
    assert_eq!(utm.to_query_string(), "utm_source=x&utm_content=footer");
}

#[test]
fn a_fully_empty_utm_set_produces_no_query_string() {
    assert_eq!(params().to_query_string(), "");
}

#[test]
fn values_are_percent_encoded() {
    let utm = UtmParams {
        campaign: Some("spring sale&more".to_owned()),
        term: Some("a/b?c=d".to_owned()),
        ..params()
    };
    assert_eq!(
        utm.to_query_string(),
        "utm_campaign=spring%20sale%26more&utm_term=a%2Fb%3Fc%3Dd"
    );
}

#[test]
fn to_json_round_trips_back_into_the_same_query_string() {
    let utm = UtmParams {
        source: Some("newsletter".to_owned()),
        campaign: Some("q3".to_owned()),
        ..params()
    };
    let json = utm.to_json().unwrap();
    let parsed: UtmParams = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.to_query_string(), utm.to_query_string());
    assert!(json.contains("\"source\":\"newsletter\""));
}

#[test]
fn link_type_strings_are_lowercase_and_match_display() {
    assert_eq!(LinkType::Redirect.as_str(), "redirect");
    assert_eq!(LinkType::Utm.as_str(), "utm");
    assert_eq!(LinkType::Both.as_str(), "both");
    assert_eq!(LinkType::Both.to_string(), "both");
}

#[test]
fn destination_type_strings_are_lowercase_and_match_display() {
    assert_eq!(DestinationType::Internal.as_str(), "internal");
    assert_eq!(DestinationType::External.as_str(), "external");
    assert_eq!(DestinationType::External.to_string(), "external");
}
