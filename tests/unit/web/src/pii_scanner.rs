// The `pii_extended` gateway scanner (REQ-030/036): formatted US SSNs and
// E.164-style phone numbers, both directions. What matters most here is the
// negative space — ids, hashes, and bare digit runs must pass, because a false
// positive on this scanner denies a customer's request.

use systemprompt::ai::SafetyScanner;
use systemprompt::models::wire::canonical::{CanonicalContent, CanonicalResponse};
use systemprompt_web_admin::gateway_safety::PiiScanner;

fn response(text: &str) -> CanonicalResponse {
    CanonicalResponse {
        id: "resp-1".to_owned(),
        model: "test-model".to_owned(),
        content: vec![CanonicalContent::Text(text.to_owned())],
        ..Default::default()
    }
}

async fn scan(text: &str) -> Vec<String> {
    PiiScanner::new()
        .scan_response_final(&response(text))
        .await
        .into_iter()
        .map(|f| f.category)
        .collect()
}

#[tokio::test]
async fn a_formatted_ssn_is_flagged_and_masked() {
    let findings = PiiScanner::new()
        .scan_response_final(&response("the ssn is 123-45-6789 apparently"))
        .await;
    assert_eq!(findings.len(), 1, "got {findings:?}");
    assert_eq!(findings[0].category, "pii_ssn");
    assert_eq!(
        findings[0].excerpt.as_deref(),
        Some("***-**-6789"),
        "the excerpt masks all but the last four"
    );
}

#[tokio::test]
async fn never_issued_ssn_ranges_pass() {
    assert!(scan("000-12-3456").await.is_empty(), "000 area");
    assert!(scan("666-12-3456").await.is_empty(), "666 area");
    assert!(scan("900-12-3456").await.is_empty(), "9xx area");
    assert!(scan("123-00-3456").await.is_empty(), "00 group");
    assert!(scan("123-45-0000").await.is_empty(), "0000 serial");
}

#[tokio::test]
async fn digits_butted_against_an_ssn_shape_pass() {
    assert!(
        scan("order 9123-45-67890 shipped").await.is_empty(),
        "part of a longer number is an id, not an SSN"
    );
}

#[tokio::test]
async fn an_e164_phone_is_flagged_but_a_bare_digit_run_is_not() {
    assert_eq!(scan("call +1 (415) 555-0100 today").await, ["pii_phone"]);
    assert!(
        scan("request 4155550100 timed out").await.is_empty(),
        "ten bare digits are an id most of the time"
    );
    assert!(
        scan("+12 is a small number").await.is_empty(),
        "too few digits after the plus"
    );
}
