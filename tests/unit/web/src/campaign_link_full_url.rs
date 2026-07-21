//! `CampaignLink::full_url` appends UTM parameters parsed from the stored
//! `utm_params` JSON, choosing `&` when the target already carries a query and
//! `?` otherwise, and falls back to the bare target when there is nothing to
//! append.

use serde_json::json;
use systemprompt_web_shared::models::CampaignLink;

// Build a `CampaignLink` from the minimal set of columns `full_url` reads.
// The remaining `Option` columns deserialize to `None`.
fn link(target_url: &str, utm_params: Option<serde_json::Value>) -> CampaignLink {
    let mut obj = json!({
        "id": "lnk_1",
        "short_code": "abc123",
        "target_url": target_url,
        "link_type": "utm",
    });
    if let Some(p) = utm_params {
        // `utm_params` is stored as a JSON *string* column, not nested JSON.
        obj["utm_params"] = json!(p.to_string());
    }
    serde_json::from_value(obj).expect("fixture columns deserialize into CampaignLink")
}

#[test]
fn no_utm_params_returns_target_unchanged() {
    let l = link("https://example.com/page", None);
    assert_eq!(l.full_url(), "https://example.com/page");
}

#[test]
fn empty_utm_object_produces_no_query_and_returns_target() {
    // All-`None` UTM fields serialize to an empty query string, so `full_url`
    // must not append a stray `?`.
    let l = link(
        "https://example.com/page",
        Some(json!({
            "source": null, "medium": null, "campaign": null,
            "term": null, "content": null,
        })),
    );
    assert_eq!(l.full_url(), "https://example.com/page");
}

#[test]
fn appends_with_question_mark_when_target_has_no_query() {
    let l = link(
        "https://example.com/page",
        Some(json!({ "source": "newsletter", "medium": "email",
                     "campaign": null, "term": null, "content": null })),
    );
    assert_eq!(
        l.full_url(),
        "https://example.com/page?utm_source=newsletter&utm_medium=email"
    );
}

#[test]
fn appends_with_ampersand_when_target_already_has_query() {
    let l = link(
        "https://example.com/page?ref=x",
        Some(json!({ "source": "newsletter", "medium": null,
                     "campaign": null, "term": null, "content": null })),
    );
    assert_eq!(
        l.full_url(),
        "https://example.com/page?ref=x&utm_source=newsletter"
    );
}

#[test]
fn url_encodes_parameter_values() {
    let l = link(
        "https://example.com/page",
        Some(json!({ "source": "a b&c", "medium": null,
                     "campaign": null, "term": null, "content": null })),
    );
    assert_eq!(l.full_url(), "https://example.com/page?utm_source=a%20b%26c");
}

#[test]
fn malformed_utm_json_falls_back_to_target() {
    // `utm_params` that is not valid `UtmParams` JSON is ignored, not fatal.
    let mut obj = json!({
        "id": "lnk_2", "short_code": "def456",
        "target_url": "https://example.com/x", "link_type": "utm",
        "utm_params": "not-json",
    });
    obj["utm_params"] = json!("not-json");
    let l: CampaignLink = serde_json::from_value(obj).expect("deserializes");
    assert_eq!(l.full_url(), "https://example.com/x");
}
