//! The content API's JSON envelopes are what browsers and the admin JS parse,
//! so their field names and derived counts are a wire contract: errors are
//! `{"error": ...}`, acknowledgements are `{"ok": true}`, and a link listing
//! reports a `total` derived from the payload rather than supplied by the
//! caller.

use systemprompt_web_extension::api::{ErrorResponse, ListLinksResponse, OkResponse};

#[test]
fn error_response_serialises_to_a_single_error_field() {
    let json = serde_json::to_value(ErrorResponse::new("link not found")).unwrap();
    assert_eq!(json, serde_json::json!({ "error": "link not found" }));
}

#[test]
fn ok_response_serialises_the_flag_it_was_given() {
    let json = serde_json::to_value(OkResponse { ok: true }).unwrap();
    assert_eq!(json, serde_json::json!({ "ok": true }));
}

#[test]
fn list_links_response_derives_total_from_the_payload() {
    let response = ListLinksResponse::new(vec!["a", "b", "c"]);
    assert_eq!(response.total, 3);
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "links": ["a", "b", "c"], "total": 3 })
    );
}

#[test]
fn an_empty_link_listing_reports_zero_not_null() {
    let json = serde_json::to_value(ListLinksResponse::<String>::new(Vec::new())).unwrap();
    assert_eq!(json, serde_json::json!({ "links": [], "total": 0 }));
}
