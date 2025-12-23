use systemprompt_mcp_infrastructure::prompts::build_deployment_guide_prompt;

#[test]
fn deployment_guide_includes_environment_name() {
    let prompt = build_deployment_guide_prompt("production", true);
    assert!(prompt.contains("production"));

    let prompt = build_deployment_guide_prompt("staging", true);
    assert!(prompt.contains("staging"));

    let prompt = build_deployment_guide_prompt("development", true);
    assert!(prompt.contains("development"));
}

#[test]
fn deployment_guide_includes_sync_steps() {
    let prompt = build_deployment_guide_prompt("production", true);

    assert!(prompt.contains("sync_status"));
    assert!(prompt.contains("sync_files"));
    assert!(prompt.contains("sync_database"));
    assert!(prompt.contains("sync_all"));
}

#[test]
fn deployment_guide_includes_rollback_when_requested() {
    let prompt = build_deployment_guide_prompt("production", true);
    assert!(prompt.contains("Rollback Procedures"));
    assert!(prompt.contains("Quick Rollback"));
    assert!(prompt.contains("Full Rollback"));
}

#[test]
fn deployment_guide_excludes_rollback_when_not_requested() {
    let prompt = build_deployment_guide_prompt("production", false);
    assert!(!prompt.contains("Rollback Procedures"));
}

#[test]
fn deployment_guide_includes_best_practices() {
    let prompt = build_deployment_guide_prompt("production", false);
    assert!(prompt.contains("Best Practices"));
    assert!(prompt.contains("dry run"));
}

#[test]
fn deployment_guide_includes_common_issues() {
    let prompt = build_deployment_guide_prompt("staging", false);
    assert!(prompt.contains("Common Issues"));
    assert!(prompt.contains("Auth errors"));
}
