//! The typed users tool must preserve the CLI's special role-filter pagination
//! rule so callers never follow a cursor that cannot return another page.

use std::path::PathBuf;

use systemprompt::identifiers::{AgentName, ContextId, McpExecutionId, SessionId, TraceId};
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_agent::CliLocation;
use systemprompt_mcp_agent::typed::{UsersHandler, UsersInput};

fn context() -> RequestContext {
    RequestContext::new(
        SessionId::new("typed-users-session"),
        TraceId::new("typed-users-trace"),
        ContextId::try_new("00000000-0000-4000-8000-00000000a11d").expect("context"),
        AgentName::try_new("typed-users-agent").expect("agent"),
        systemprompt::identifiers::Actor::anonymous(systemprompt::identifiers::UserId::generate()),
    )
}

fn fake_cli(dir: &tempfile::TempDir) -> CliLocation {
    let path = dir.path().join("systemprompt");
    std::fs::write(
        &path,
        "#!/bin/sh\nprintf '%s' '[{\"id\":\"user-a\",\"roles\":[\"admin\"]},{\"id\":\"user-b\",\"roles\":[\"user\"]}]'\n",
    )
    .expect("write CLI fixture");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
        .expect("make fixture executable");
    CliLocation {
        bin: path,
        workdir: PathBuf::from(dir.path()),
    }
}

#[tokio::test]
async fn users_handler_emits_an_offset_cursor_for_a_full_unfiltered_page() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cli = fake_cli(&dir);
    let handler = UsersHandler {
        cli: &cli,
        token: "test-token",
    };

    let (page, summary) = handler
        .handle(
            UsersInput {
                limit: 2,
                offset: 4,
                role: String::new(),
                status: "active".into(),
            },
            &context(),
            &McpExecutionId::new("typed-users-exec"),
        )
        .await
        .expect("read users page");

    assert_eq!(
        page.command,
        "admin users list --limit 2 --offset 4 --status active"
    );
    assert_eq!(page.columns, ["id", "roles"]);
    assert_eq!(page.returned, 2);
    assert_eq!(page.next_cursor, "6");
    assert!(page.hint.contains("More users may follow"));
    assert!(summary.contains("next_cursor=6"));
}

#[tokio::test]
async fn users_handler_does_not_offer_a_cursor_for_role_filtered_results() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cli = fake_cli(&dir);
    let handler = UsersHandler {
        cli: &cli,
        token: "test-token",
    };

    let (page, summary) = handler
        .handle(
            UsersInput {
                limit: 2,
                offset: 4,
                role: "admin".into(),
                status: String::new(),
            },
            &context(),
            &McpExecutionId::new("typed-users-role-exec"),
        )
        .await
        .expect("read role-filtered users");

    assert_eq!(
        page.command,
        "admin users list --limit 2 --offset 4 --role admin"
    );
    assert_eq!(page.returned, 2);
    assert!(page.next_cursor.is_empty());
    assert!(page.hint.contains("Last page"));
    assert!(!summary.contains("next_cursor"));
}
