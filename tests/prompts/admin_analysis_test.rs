use systemprompt_admin::prompts::build_admin_analysis_prompt;

#[test]
fn admin_analysis_prompt_contains_focus_area() {
    let prompt = build_admin_analysis_prompt("logs", "24h");

    assert!(prompt.contains("logs"));
    assert!(prompt.contains("24h"));
}

#[test]
fn admin_analysis_prompt_contains_structure_sections() {
    let prompt = build_admin_analysis_prompt("all", "7d");

    assert!(prompt.contains("Executive Summary"));
    assert!(prompt.contains("Detailed Analysis"));
    assert!(prompt.contains("Recommendations"));
}

#[test]
fn admin_analysis_prompt_handles_all_focus_areas() {
    let areas = ["logs", "database", "system", "users", "all"];

    for area in &areas {
        let prompt = build_admin_analysis_prompt(area, "24h");
        assert!(!prompt.is_empty());
        assert!(prompt.contains(area));
    }
}

#[test]
fn admin_analysis_prompt_handles_all_time_periods() {
    let periods = ["1h", "24h", "7d", "30d"];

    for period in &periods {
        let prompt = build_admin_analysis_prompt("all", period);
        assert!(!prompt.is_empty());
        assert!(prompt.contains(period));
    }
}

#[test]
fn admin_analysis_prompt_contains_tool_instructions() {
    let prompt = build_admin_analysis_prompt("all", "24h");

    assert!(prompt.contains("get_logs"));
    assert!(prompt.contains("db_admin"));
    assert!(prompt.contains("system_status"));
    assert!(prompt.contains("user_activity"));
}
