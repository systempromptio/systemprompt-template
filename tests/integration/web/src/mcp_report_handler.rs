//! End-to-end report-handler coverage through a stand-in CLI. The report has
//! four fixed commands: callers choose only the lookback period, and one
//! failed source must leave the other deterministic report data available.

use std::path::PathBuf;

use systemprompt::identifiers::{AgentName, ContextId, McpExecutionId, SessionId, TraceId};
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_agent::CliLocation;
use systemprompt_mcp_agent::reports::{ReportHandler, ReportInput, ReportKind};

fn shell_quote(path: &std::path::Path) -> String {
    // `/bin/sh` sees the generated path as syntax. Quote embedded apostrophes
    // too, so test paths with whitespace or shell metacharacters remain data.
    format!("'{}'", path.to_string_lossy().replace('\'', "'\"'\"'"))
}

fn context() -> RequestContext {
    RequestContext::new(
        SessionId::new("report-session"),
        TraceId::new("report-trace"),
        ContextId::try_new("00000000-0000-4000-8000-00000000a11c").expect("context id"),
        AgentName::try_new("report-agent").expect("agent name"),
        systemprompt::identifiers::Actor::anonymous(systemprompt::identifiers::UserId::generate()),
    )
}

fn fake_cli(dir: &tempfile::TempDir, body: &str) -> CliLocation {
    let path = dir.path().join("systemprompt");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write CLI fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("make fixture executable");
    }
    CliLocation {
        bin: path,
        workdir: PathBuf::from(dir.path()),
    }
}

async fn run(cli: &CliLocation) -> (systemprompt_mcp_agent::reports::ReportOutput, String) {
    run_with_days(cli, 7)
        .await
        .expect("report returns an output even when individual sources fail")
}

async fn run_with_days(
    cli: &CliLocation,
    days: u16,
) -> Result<(systemprompt_mcp_agent::reports::ReportOutput, String), rmcp::ErrorData> {
    ReportHandler {
        cli,
        token: "report-test-token",
    }
    .handle(
        ReportInput {
            report: ReportKind::Costs,
            days,
        },
        &context(),
        &McpExecutionId::new("report-exec"),
    )
    .await
}

#[tokio::test]
async fn report_handler_runs_only_fixed_cost_commands_and_aggregates_their_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let commands = dir.path().join("commands");
    let cli = fake_cli(
        &dir,
        &format!(
            r#"printf '%s\n' "$*" >> {commands}
case "$*" in
  "analytics costs summary --since 7d") printf '%s' '[{{"heading":"total_cost_microdollars","content":2500000}},{{"heading":"total_requests","content":4}}]' ;;
  "analytics requests models --since 7d --limit 100") printf '%s' '[{{"model":"model-a","total_cost_microdollars":500000}}]' ;;
  "analytics costs trends --since 7d") printf '%s' '{{"labels":["2026-09-01"],"datasets":[{{"label":"cost_usd","data":[2.5]}}]}}' ;;
  "analytics sessions stats --since 7d") printf '%s' '[{{"heading":"total_tokens","content":99}}]' ;;
  *) exit 99 ;;
esac"#,
            commands = shell_quote(&commands)
        ),
    );

    let (report, summary) = run(&cli).await;

    assert!(report.complete);
    assert_eq!(report.sources.len(), 4);
    assert_eq!(report.tables.len(), 4);
    assert_eq!(report.metrics["total_cost_microdollars"], 2_500_000);
    assert_eq!(report.metrics["total_requests"], 4);
    assert_eq!(report.metrics["total_tokens"], 99);
    assert_eq!(report.tables[1].rows[0]["total_cost_usd"], 0.5);
    assert_eq!(report.tables[2].rows[0]["period"], "2026-09-01");
    assert!(summary.contains("Sources loaded"));
    assert_eq!(
        std::fs::read_to_string(commands)
            .expect("command log")
            .lines()
            .collect::<Vec<_>>(),
        [
            "analytics costs summary --since 7d",
            "analytics requests models --since 7d --limit 100",
            "analytics costs trends --since 7d",
            "analytics sessions stats --since 7d",
        ]
    );
}

#[tokio::test]
async fn report_handler_returns_partial_data_when_one_fixed_source_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cli = fake_cli(
        &dir,
        r#"case "$*" in
  "analytics costs summary --since 7d") printf '%s' '[{"heading":"total_requests","content":8}]' ;;
  "analytics requests models --since 7d --limit 100") echo 'upstream unavailable' >&2; exit 23 ;;
  "analytics costs trends --since 7d") printf '%s' '{"labels":["2026-09-01"],"datasets":[{"label":"cost_usd","data":[1.25]}]}' ;;
  "analytics sessions stats --since 7d") printf '%s' '[{"heading":"total_tokens","content":42}]' ;;
  *) exit 99 ;;
esac"#,
    );

    let (report, summary) = run(&cli).await;

    assert!(!report.complete);
    assert_eq!(report.sources.len(), 4);
    assert_eq!(
        report
            .sources
            .iter()
            .filter(|source| source.complete)
            .count(),
        3
    );
    assert_eq!(report.tables.len(), 3);
    assert_eq!(report.metrics["total_requests"], 8);
    assert_eq!(report.metrics["total_tokens"], 42);
    assert_eq!(report.tables[1].rows[0]["cost_usd"], 1.25);
    let failed = report
        .sources
        .iter()
        .find(|source| source.source == "analytics requests models --since 7d --limit 100")
        .expect("the failed source is represented in the dashboard");
    assert!(!failed.complete);
    assert!(
        failed
            .warning
            .as_deref()
            .is_some_and(|warning| warning.contains("exit 23"))
    );
    assert!(summary.contains("Partial data; inspect source warnings"));
}

#[tokio::test]
async fn report_handler_accepts_both_documented_lookback_boundaries() {
    for days in [1, 90] {
        let dir = tempfile::tempdir().expect("tempdir");
        let commands = dir.path().join("commands");
        let cli = fake_cli(
            &dir,
            &format!(
                r#"printf '%s\n' "$*" >> {commands}
printf '%s' '[]'"#,
                commands = shell_quote(&commands)
            ),
        );

        let (report, _) = run_with_days(&cli, days).await.expect("boundary is valid");

        assert!(report.complete);
        assert_eq!(report.sources.len(), 4);
        let command_log = std::fs::read_to_string(commands).expect("command log");
        assert_eq!(command_log.lines().count(), 4);
        assert!(
            command_log
                .lines()
                .all(|command| command.contains(&format!("--since {days}d"))),
            "each source must receive the requested bounded lookback: {command_log}"
        );
    }
}

#[tokio::test]
async fn invalid_lookback_is_refused_before_the_cli_is_spawned() {
    for days in [0, 91] {
        let dir = tempfile::tempdir().expect("tempdir");
        let marker = dir.path().join("spawned");
        let cli = fake_cli(&dir, &format!("touch {}", shell_quote(&marker)));

        let error = run_with_days(&cli, days)
            .await
            .expect_err("out-of-range lookback must fail before source reads");

        assert!(error.message.contains("days must be between 1 and 90"));
        assert!(
            !marker.exists(),
            "invalid input must not spawn a CLI source for {days} days"
        );
    }
}
