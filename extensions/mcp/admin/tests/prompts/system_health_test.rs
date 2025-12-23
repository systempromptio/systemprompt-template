use systemprompt_admin::prompts::build_system_health_prompt;

#[test]
fn system_health_prompt_includes_recommendations_when_requested() {
    let prompt = build_system_health_prompt(true);

    assert!(prompt.contains("Recommendations"));
    assert!(prompt.contains("Immediate Actions"));
}

#[test]
fn system_health_prompt_excludes_recommendations_when_not_requested() {
    let prompt = build_system_health_prompt(false);

    assert!(!prompt.contains("Immediate Actions"));
}

#[test]
fn system_health_prompt_contains_diagnostic_sequence() {
    let prompt = build_system_health_prompt(true);

    assert!(prompt.contains("System Status Check"));
    assert!(prompt.contains("Database Health Assessment"));
    assert!(prompt.contains("Log Analysis"));
    assert!(prompt.contains("User Activity Review"));
}

#[test]
fn system_health_prompt_contains_report_format() {
    let prompt = build_system_health_prompt(true);

    assert!(prompt.contains("Health Report"));
    assert!(prompt.contains("Resource Status"));
    assert!(prompt.contains("Critical Issues"));
    assert!(prompt.contains("Warning Indicators"));
    assert!(prompt.contains("Performance Metrics"));
}

#[test]
fn system_health_prompt_is_not_empty() {
    let prompt_with_recs = build_system_health_prompt(true);
    let prompt_without_recs = build_system_health_prompt(false);

    assert!(!prompt_with_recs.is_empty());
    assert!(!prompt_without_recs.is_empty());
    assert!(prompt_with_recs.len() > prompt_without_recs.len());
}
