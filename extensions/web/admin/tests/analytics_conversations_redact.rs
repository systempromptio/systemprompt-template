use systemprompt_web_admin::repositories::analytics_grp::conversations::redact_text;

#[test]
fn redact_aws_key() {
    let (out, n) = redact_text("here is AKIAIOSFODNN7EXAMPLE in text");
    assert_eq!(n, 1);
    assert!(out.contains("[REDACTED:aws_access_key]"));
    assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn redact_anthropic_key() {
    let (out, n) = redact_text("call sk-ant-api03-abc and also AIzaSyAbCdEfG please");
    assert_eq!(n, 2);
    assert!(out.contains("[REDACTED:anthropic_api_key]"));
    assert!(out.contains("[REDACTED:google_api_key]"));
}

#[test]
fn redact_no_op_on_clean_text() {
    let (out, n) = redact_text("hello world, no secrets here");
    assert_eq!(n, 0);
    assert_eq!(out, "hello world, no secrets here");
}
