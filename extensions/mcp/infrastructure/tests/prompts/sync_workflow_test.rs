use systemprompt_mcp_infrastructure::prompts::build_sync_workflow_prompt;

#[test]
fn sync_workflow_includes_direction() {
    let prompt = build_sync_workflow_prompt("push", "all");
    assert!(prompt.contains("push"));
    assert!(prompt.contains("uploading local changes to cloud"));

    let prompt = build_sync_workflow_prompt("pull", "all");
    assert!(prompt.contains("pull"));
    assert!(prompt.contains("downloading cloud state to local"));
}

#[test]
fn sync_workflow_includes_scope() {
    let prompt = build_sync_workflow_prompt("push", "files");
    assert!(prompt.contains("files"));
    assert!(prompt.contains("File Sync Guidelines"));

    let prompt = build_sync_workflow_prompt("push", "database");
    assert!(prompt.contains("database"));
    assert!(prompt.contains("Database Sync Guidelines"));

    let prompt = build_sync_workflow_prompt("push", "all");
    assert!(prompt.contains("all"));
    assert!(prompt.contains("Full Sync Guidelines"));
}

#[test]
fn sync_workflow_includes_dry_run_section() {
    let prompt = build_sync_workflow_prompt("push", "all");
    assert!(prompt.contains("Dry Run Phase"));
    assert!(prompt.contains("dry_run: true"));
}

#[test]
fn sync_workflow_includes_verification_section() {
    let prompt = build_sync_workflow_prompt("pull", "database");
    assert!(prompt.contains("Post-Sync Verification"));
}

#[test]
fn sync_workflow_includes_error_handling() {
    let prompt = build_sync_workflow_prompt("push", "files");
    assert!(prompt.contains("Error Handling"));
    assert!(prompt.contains("connectivity"));
}

#[test]
fn sync_workflow_includes_best_practices() {
    let prompt = build_sync_workflow_prompt("pull", "all");
    assert!(prompt.contains("Best Practices"));
    assert!(prompt.contains("Consistent direction"));
}

#[test]
fn sync_workflow_files_scope_mentions_yaml() {
    let prompt = build_sync_workflow_prompt("push", "files");
    assert!(prompt.contains("*.yml"));
    assert!(prompt.contains("*.yaml"));
}

#[test]
fn sync_workflow_database_scope_mentions_tables() {
    let prompt = build_sync_workflow_prompt("push", "database");
    assert!(prompt.contains("agents"));
    assert!(prompt.contains("skills"));
    assert!(prompt.contains("contexts"));
}
