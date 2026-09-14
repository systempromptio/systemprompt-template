//! The shipped governance chain under `governance.mode: warn`, end to end:
//! `services/governance/config.yaml` is parsed by the same loader the server
//! uses, built into the same `GovernanceEngine`, and driven with the inputs
//! that were denying real traffic before the mode existed. Every stage must
//! still find what it found, and none of them may refuse.
//!
//! The write side is exercised too: each warn verdict is recorded through
//! `record_decision` into a throwaway database and read back with the same
//! rollup query `infra logs governance report` and `/admin/governance/warnings`
//! use, so "warns are logged" is proven against the table rather than assumed
//! from the verdict.
//!
//! Bare entropy in prose is a nonblocking observation. Recognized credentials
//! on tool surfaces still produce a recorded warning under the shipped mode.

use std::path::PathBuf;

use serde_json::json;
use systemprompt::identifiers::{CallId, McpToolName, SessionId, UserId};
use systemprompt_security::authz::list_governance_warnings;
use systemprompt_security::authz::types::DecisionTag;
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, AuditOrigin, AuditTarget, ChainEntryResult, DecisionAudit, Evaluation,
    GovernanceConfig, GovernanceEngine, GovernedInput, GovernedTarget, McpToolInput, PolicyContext,
    PolicyMode, PrincipalSnapshot, record_decision,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

// Why: a fixed ancestor depth silently points at the wrong directory the
// moment a crate moves; `repo_root` climbs until the repository's own
// markers appear and fails loudly when they never do.
fn shipped_config_path() -> PathBuf {
    astound_test_common::repo_path("services/governance/config.yaml")
}

fn shipped_config() -> GovernanceConfig {
    let yaml = std::fs::read_to_string(shipped_config_path()).expect("read the shipped chain");
    GovernanceConfig::parse(&yaml).expect("the shipped governance config parses")
}

fn shipped_engine() -> GovernanceEngine {
    GovernanceEngine::from_config(&shipped_config()).expect("shipped governance config is valid")
}

// 32 random bytes as unpadded base64: no vendor prefix, decodes to nothing
// readable, clears the entropy ceiling. Exactly the shape the backstop exists
// to catch, and nothing else in the chain matches it.
const PREFIXLESS_KEY: &str = "oJXyD5OVZQz5OAuO2yJKaySKHpJOj9CuLhqUkqMwXxg";

const GITHUB_TOKEN: &str = "ghp_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789";

struct Call<'a> {
    target: GovernedTarget,
    input: GovernedInput,
    scope: AccessScope,
    user: &'a UserId,
    session: &'a SessionId,
    call_id: CallId,
}

impl<'a> Call<'a> {
    fn prompt(text: &str, user: &'a UserId, session: &'a SessionId) -> Self {
        Self {
            target: GovernedTarget::Prompt,
            input: GovernedInput::prompt_text(text.to_owned()),
            scope: AccessScope::User,
            user,
            session,
            call_id: CallId::new(unique("call")),
        }
    }

    fn tool(name: &str, args: serde_json::Value, user: &'a UserId, session: &'a SessionId) -> Self {
        Self {
            target: GovernedTarget::Tool {
                tool: McpToolName::new(name),
            },
            input: GovernedInput::tool_arguments(McpToolInput::new(args)),
            scope: AccessScope::User,
            user,
            session,
            call_id: CallId::new(unique("call")),
        }
    }

    fn evaluate(&self, engine: &GovernanceEngine) -> Evaluation {
        engine.evaluate(&PolicyContext {
            target: self.target.clone(),
            agent_scope: AgentScope::User {
                user_id: self.user.clone(),
            },
            access_scope: self.scope,
            session_id: self.session,
            user_id: self.user,
            input: &self.input,
            call_id: &self.call_id,
        })
    }

    fn audit(&self, evaluation: Evaluation) -> DecisionAudit {
        DecisionAudit {
            id: unique("dec"),
            call_id: self.call_id.as_str().to_owned(),
            origin: AuditOrigin::Governed,
            decision: evaluation.decision,
            principal: PrincipalSnapshot {
                user_id: self.user.clone(),
                session_id: self.session.clone(),
                agent_session: None,
                agent_id: None,
                agent_scope: self.scope,
                client_id: None,
                claimed: None,
            },
            target: AuditTarget {
                tool_name: self.target.as_str().to_owned(),
                plugin_id: None,
            },
            chain: evaluation.chain,
            approver: None,
            act_chain: Vec::new(),
            context_id: None,
            trace_id: Some(unique("trace")),
        }
    }
}

fn warned_policies(evaluation: &Evaluation) -> Vec<&str> {
    evaluation
        .chain
        .iter()
        .filter(|e| e.result == ChainEntryResult::Warn)
        .map(|e| e.policy_id.as_str())
        .collect()
}

fn failed_policies(evaluation: &Evaluation) -> Vec<&str> {
    evaluation
        .chain
        .iter()
        .filter(|e| e.result == ChainEntryResult::Fail)
        .map(|e| e.policy_id.as_str())
        .collect()
}

#[test]
fn the_shipped_chain_has_every_stage_on_and_every_stage_warning() {
    let config = shipped_config();
    assert!(config.enabled, "the master switch is on");
    assert_eq!(
        config.mode,
        PolicyMode::Warn,
        "the chain-wide default is warn"
    );
    let ids: Vec<&str> = config.policies.iter().map(|p| p.id.as_str()).collect();
    for stage in ["secret_scan", "scope_check", "tool_blocklist", "rate_limit"] {
        let policy = config
            .policies
            .iter()
            .find(|p| p.id == stage)
            .unwrap_or_else(|| panic!("{stage} is declared; chain has {ids:?}"));
        assert!(policy.enabled, "{stage} is enabled — warn is not off");
        assert!(
            policy.mode.is_warn(),
            "{stage} inherits warn and does not override it"
        );
    }
}

#[test]
fn the_entropy_backstop_is_on_in_the_shipped_file() {
    let config = shipped_config();
    let secret_scan = config
        .policies
        .iter()
        .find(|p| p.id == "secret_scan")
        .expect("secret_scan declared");
    let entropy = secret_scan
        .params
        .get("entropy")
        .expect("an explicit entropy block");
    assert_eq!(
        entropy.get("enabled").and_then(serde_yaml::Value::as_bool),
        Some(true),
        "warn mode is the answer to the backstop's false positives, not switching it off"
    );
}

#[test]
fn bare_entropy_in_a_prompt_is_a_nonblocking_observation() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::prompt(
        &format!("deploy with token {PREFIXLESS_KEY} please"),
        &user,
        &session,
    );
    let evaluation = call.evaluate(&engine);
    assert_eq!(evaluation.decision.tag(), DecisionTag::Allow);
    assert!(warned_policies(&evaluation).is_empty());
    assert!(failed_policies(&evaluation).is_empty());
}

#[test]
fn bare_entropy_in_tool_arguments_is_nonblocking() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::tool(
        "Bash",
        json!({ "command": format!("curl -H 'Authorization: {PREFIXLESS_KEY}' https://x") }),
        &user,
        &session,
    );

    let evaluation = call.evaluate(&engine);

    assert_eq!(evaluation.decision.tag(), DecisionTag::Allow);
    assert!(warned_policies(&evaluation).is_empty());
}

// Why: these were the false positives. A commit SHA is hex, a macOS $TMPDIR is
// a path, an SRI digest is allowlisted; none of them is key material and none
// of them may even warn, or the report fills with noise it was built to
// remove.
#[test]
fn the_backstops_known_false_positives_do_not_even_warn() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let benign = [
        "commit 3f2a9c1d8e7b6a5f4c3d2e1f0a9b8c7d6e5f4a3b touched two files",
        "cwd is /private/var/folders/x1/9kQm2Lp7R3vZt8Wn4Yb6c0000gn/T/ for this run",
        "<script integrity=\"sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC\">",
    ];
    for text in benign {
        let call = Call::prompt(text, &user, &session);
        let evaluation = call.evaluate(&engine);
        assert_eq!(
            evaluation.decision.tag(),
            DecisionTag::Allow,
            "{text:?} is not key material and must be a clean allow, got {:?}",
            evaluation.decision
        );
    }
}

#[test]
fn a_configured_provider_key_is_caught_and_is_a_warn() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::tool(
        "Write",
        json!({ "path": ".env", "content": format!("GITHUB_TOKEN={GITHUB_TOKEN}") }),
        &user,
        &session,
    );

    let evaluation = call.evaluate(&engine);

    assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
    assert_eq!(warned_policies(&evaluation), vec!["secret_scan"]);
}

#[test]
fn an_admin_only_tool_from_a_user_is_a_warn_from_scope_check() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::tool("mcp__systemprompt__list_users", json!({}), &user, &session);

    let evaluation = call.evaluate(&engine);

    assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
    assert_eq!(warned_policies(&evaluation), vec!["scope_check"]);
}

#[test]
fn a_blocklisted_tool_name_is_a_warn_from_tool_blocklist() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::tool("delete_branch", json!({ "name": "old" }), &user, &session);

    let evaluation = call.evaluate(&engine);

    assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
    assert_eq!(warned_policies(&evaluation), vec!["tool_blocklist"]);
}

// Why: the shipped window is 300 calls a minute per session. Under enforce the
// 301st call is refused; under warn it is recorded and allowed, and every
// call after it in the same window is another warn row rather than a refusal.
#[test]
fn an_exhausted_rate_limit_window_is_a_warn_from_rate_limit() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));

    let mut last = None;
    for _ in 0..301 {
        let call = Call::prompt("hello", &user, &session);
        last = Some(call.evaluate(&engine));
    }
    let evaluation = last.expect("301 evaluations ran");

    assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
    assert_eq!(warned_policies(&evaluation), vec!["rate_limit"]);
    assert!(evaluation.decision.permits());
}

// Why: a deny halts the chain, so under enforce only the first stage to fire
// is ever recorded. Warn does not halt: a call that trips three stages is
// recorded against all three, which is what makes the report honest about
// what enforcement would cost.
#[test]
fn a_warn_does_not_halt_the_chain_so_every_finding_on_the_call_is_recorded() {
    let engine = shipped_engine();
    let user = UserId::new(unique("user"));
    let session = SessionId::new(unique("session"));
    let call = Call::tool(
        "mcp__systemprompt__delete_user",
        json!({ "id": "u1", "token": GITHUB_TOKEN }),
        &user,
        &session,
    );

    let evaluation = call.evaluate(&engine);

    assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
    let mut warned = warned_policies(&evaluation);
    warned.sort_unstable();
    assert_eq!(warned, vec!["scope_check", "secret_scan", "tool_blocklist"]);
    assert!(
        !evaluation
            .chain
            .iter()
            .any(|e| e.result == ChainEntryResult::Skip),
        "no stage was skipped as 'already halted'"
    );
}

#[tokio::test]
async fn every_warn_is_written_to_governance_decisions_and_rolls_up_by_policy() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let engine = shipped_engine();
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("warn")).await;
    let session = SessionId::new(unique("session"));

    let calls = [
        Call::tool(
            "Write",
            json!({"content": format!("GITHUB_TOKEN={GITHUB_TOKEN}")}),
            &user,
            &session,
        ),
        Call::tool("mcp__systemprompt__list_users", json!({}), &user, &session),
        Call::tool("drop_table", json!({ "t": "x" }), &user, &session),
    ];
    for call in &calls {
        let evaluation = call.evaluate(&engine);
        assert_eq!(evaluation.decision.tag(), DecisionTag::Warn);
        record_decision(&db.pool, &call.audit(evaluation))
            .await
            .expect("the warn row is written");
    }

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT decision, policy, reason FROM governance_decisions
         WHERE user_id = $1 ORDER BY policy",
    )
    .bind(user.as_str())
    .fetch_all(&*db.pool)
    .await
    .expect("read the rows back");
    assert_eq!(rows.len(), 3, "one row per call, none dropped");
    for (decision, policy, reason) in &rows {
        assert_eq!(decision, "warn", "{policy}: the verdict column says warn");
        assert!(
            !reason.is_empty(),
            "{policy}: the reason a deny would have carried is kept"
        );
    }
    let policies: Vec<&str> = rows.iter().map(|r| r.1.as_str()).collect();
    assert_eq!(
        policies,
        vec!["scope_check", "secret_scan", "tool_blocklist"],
        "the policy column names the stage that warned, never default_allow"
    );

    let rollup = list_governance_warnings(&db.pool, None, 100)
        .await
        .expect("the report query runs");
    let ours: Vec<_> = rollup
        .iter()
        .filter(|r| r.user_id == user.as_str())
        .collect();
    assert_eq!(
        ours.len(),
        3,
        "the report sees every warned policy for this user"
    );
    assert!(
        ours.iter()
            .all(|r| r.count == 1 && !r.example_reason.is_empty())
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_clean_call_is_still_recorded_as_an_allow_not_a_warn() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let engine = shipped_engine();
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("clean")).await;
    let session = SessionId::new(unique("session"));
    let call = Call::tool("Read", json!({ "path": "README.md" }), &user, &session);

    let evaluation = call.evaluate(&engine);
    assert_eq!(evaluation.decision.tag(), DecisionTag::Allow);
    record_decision(&db.pool, &call.audit(evaluation))
        .await
        .expect("the allow row is written");

    let (decision, policy): (String, String) =
        sqlx::query_as("SELECT decision, policy FROM governance_decisions WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("read the row back");
    assert_eq!(decision, "allow");
    assert_eq!(
        policy, "default_allow",
        "warn mode does not relabel clean traffic"
    );
    let rollup = list_governance_warnings(&db.pool, None, 100)
        .await
        .expect("the report query runs");
    assert!(
        !rollup.iter().any(|r| r.user_id == user.as_str()),
        "an allow never appears in the warn report"
    );

    db.cleanup().await;
}
