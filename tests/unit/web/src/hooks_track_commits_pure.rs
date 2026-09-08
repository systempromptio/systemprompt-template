#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

//! `/hooks/track` commit detection: the Bash-command gate and the git stdout
//! parser. These pin the porcelain formats the parser understands — the
//! `[branch hash] subject` summary line and the `N files changed, …` stats
//! line — and the shapes it must survive (missing stats, chained commands,
//! bare-string responses).

use systemprompt_web_admin::test_support::{
    is_commit_command, parse_commit_stdout, response_stdout,
};

#[test]
fn commit_command_detection() {
    assert!(is_commit_command("git commit -m 'x'"));
    assert!(is_commit_command("git -C /some/dir commit -am msg"));
    assert!(is_commit_command(
        "git add -A && git commit -m 'x' && git push"
    ));
    assert!(!is_commit_command("git commit --dry-run -m 'x'"));
    assert!(!is_commit_command("git status"));
    assert!(!is_commit_command("echo commit"));
    assert!(!is_commit_command("commit git"));
}

#[test]
fn parses_summary_and_stats_line() {
    let stdout = "[main abc1234] Add the analytics dashboard\n \
                  3 files changed, 120 insertions(+), 4 deletions(-)\n";
    let commits = parse_commit_stdout(stdout);
    assert_eq!(commits.len(), 1);
    let c = &commits[0];
    assert_eq!(c.branch.as_deref(), Some("main"));
    assert_eq!(c.hash, "abc1234");
    assert_eq!(c.message, "Add the analytics dashboard");
    assert_eq!(c.files_changed, Some(3));
    assert_eq!(c.insertions, Some(120));
    assert_eq!(c.deletions, Some(4));
}

#[test]
fn parses_singular_clause_and_missing_stats() {
    let commits = parse_commit_stdout("[fix/x deadbee] tiny\n 1 file changed, 1 insertion(+)\n");
    assert_eq!(commits[0].files_changed, Some(1));
    assert_eq!(commits[0].insertions, Some(1));
    assert_eq!(commits[0].deletions, None);

    let quiet = parse_commit_stdout("[main 1234567] quiet commit\n");
    assert_eq!(quiet.len(), 1);
    assert_eq!(quiet[0].files_changed, None);
}

#[test]
fn parses_root_commit_and_detached_head() {
    let root = parse_commit_stdout(
        "[main (root-commit) 0f1e2d3] first\n 1 file changed, 1 insertion(+)\n",
    );
    assert_eq!(root[0].branch.as_deref(), Some("main"));
    assert_eq!(root[0].hash, "0f1e2d3");

    let detached = parse_commit_stdout("[detached HEAD 9abcdef] fix\n");
    assert_eq!(detached[0].branch.as_deref(), Some("detached HEAD"));
    assert_eq!(detached[0].hash, "9abcdef");
}

#[test]
fn chained_commands_yield_one_commit_per_summary_line() {
    let stdout = "[main aaaa111] first\n 1 file changed, 2 insertions(+)\n\
                  [main bbbb222] second\n 2 files changed, 3 insertions(+), 1 deletion(-)\n";
    let commits = parse_commit_stdout(stdout);
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].insertions, Some(2));
    assert_eq!(commits[1].hash, "bbbb222");
    assert_eq!(commits[1].deletions, Some(1));
}

#[test]
fn non_commit_output_parses_to_nothing() {
    assert!(parse_commit_stdout("On branch main\nnothing to commit\n").is_empty());
    // A bracketed line whose last token is not a hex hash is not a summary.
    assert!(parse_commit_stdout("[warn] something happened\n").is_empty());
}

#[test]
fn response_stdout_handles_both_shapes() {
    assert_eq!(
        response_stdout(&serde_json::json!({"stdout": "out", "stderr": "err"})),
        "out"
    );
    assert_eq!(
        response_stdout(&serde_json::json!("bare string")),
        "bare string"
    );
    assert_eq!(response_stdout(&serde_json::json!(42)), "");
}
