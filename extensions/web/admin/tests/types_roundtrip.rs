//! Serde, `Display`, and `FromStr` contracts for the admin value types.
//!
//! These shapes cross a wire (Postgres text columns, hook payloads posted by
//! Claude Code, template JSON), so their string encodings are the contract —
//! a renamed variant is a silently dropped audit row, not a compile error.

use systemprompt_web_admin::activity::{ActivityAction, ActivityCategory, ActivityEntity};
use systemprompt_web_admin::types::access_control::{AccessControlRule, AccessDecision};
use systemprompt_web_admin::types::hooks_export::{
    HookEventType, HookHandler, HooksFile, HttpHook, MatcherGroup,
};
use systemprompt_web_admin::types::webhook::{HookEvent, HookEventPayload, ToolInputSummary};
use systemprompt_web_admin::types::{DashboardQuery, GatewayRouteView};

const ACTIVITY_CATEGORIES: &[(ActivityCategory, &str)] = &[
    (ActivityCategory::Login, "login"),
    (ActivityCategory::Session, "session"),
    (ActivityCategory::Prompt, "prompt"),
    (ActivityCategory::SkillUsage, "skill_usage"),
    (ActivityCategory::MarketplaceEdit, "marketplace_edit"),
    (ActivityCategory::MarketplaceConnect, "marketplace_connect"),
    (ActivityCategory::UserManagement, "user_management"),
    (ActivityCategory::ToolUsage, "tool_usage"),
    (ActivityCategory::Error, "error"),
    (ActivityCategory::AgentResponse, "agent_response"),
    (ActivityCategory::Notification, "notification"),
    (ActivityCategory::TaskCompletion, "task_completion"),
    (ActivityCategory::Compaction, "compaction"),
    (ActivityCategory::McpAccess, "mcp_access"),
];

const ACTIVITY_ACTIONS: &[(ActivityAction, &str)] = &[
    (ActivityAction::LoggedIn, "logged_in"),
    (ActivityAction::Started, "started"),
    (ActivityAction::Ended, "ended"),
    (ActivityAction::Submitted, "submitted"),
    (ActivityAction::Used, "used"),
    (ActivityAction::Created, "created"),
    (ActivityAction::Updated, "updated"),
    (ActivityAction::Deleted, "deleted"),
    (ActivityAction::Imported, "imported"),
    (ActivityAction::Uploaded, "uploaded"),
    (ActivityAction::Restored, "restored"),
    (ActivityAction::Authenticated, "authenticated"),
    (ActivityAction::Rejected, "rejected"),
];

const ACTIVITY_ENTITIES: &[(ActivityEntity, &str)] = &[
    (ActivityEntity::Session, "session"),
    (ActivityEntity::Skill, "skill"),
    (ActivityEntity::Plugin, "plugin"),
    (ActivityEntity::Hook, "hook"),
    (ActivityEntity::McpServer, "mcp_server"),
    (ActivityEntity::Marketplace, "marketplace"),
    (ActivityEntity::User, "user"),
    (ActivityEntity::Prompt, "prompt"),
    (ActivityEntity::Agent, "agent"),
    (ActivityEntity::Tool, "tool"),
    (ActivityEntity::GatewayRoute, "gateway_route"),
];

const HOOK_EVENT_TYPES: &[HookEventType] = &[
    HookEventType::SessionStart,
    HookEventType::SessionEnd,
    HookEventType::UserPromptSubmit,
    HookEventType::PreToolUse,
    HookEventType::PostToolUse,
    HookEventType::PostToolUseFailure,
    HookEventType::PermissionRequest,
    HookEventType::Stop,
    HookEventType::SubagentStart,
    HookEventType::SubagentStop,
    HookEventType::TaskCompleted,
    HookEventType::TeammateIdle,
    HookEventType::Notification,
    HookEventType::ConfigChange,
    HookEventType::WorktreeCreate,
    HookEventType::WorktreeRemove,
    HookEventType::PreCompact,
    HookEventType::InstructionsLoaded,
];

#[test]
fn activity_category_display_matches_serde() {
    for (variant, slug) in ACTIVITY_CATEGORIES {
        assert_eq!(variant.to_string(), *slug);
        assert_eq!(variant.as_ref(), *slug);
        assert_eq!(
            serde_json::to_value(variant).expect("serialize category"),
            serde_json::Value::String((*slug).to_owned()),
        );
    }
}

#[test]
fn activity_category_parses_every_slug_it_emits() {
    for (variant, slug) in ACTIVITY_CATEGORIES {
        let parsed: ActivityCategory = slug.parse().expect("slug round-trips");
        assert_eq!(parsed, *variant);
    }
}

#[test]
fn activity_category_rejects_unknown_slug() {
    let err = "not_a_category"
        .parse::<ActivityCategory>()
        .expect_err("unknown slug rejected");
    assert!(err.contains("not_a_category"), "{err}");
}

#[test]
fn activity_action_display_matches_serde() {
    for (variant, slug) in ACTIVITY_ACTIONS {
        assert_eq!(variant.to_string(), *slug);
        assert_eq!(
            serde_json::to_value(variant).expect("serialize action"),
            serde_json::Value::String((*slug).to_owned()),
        );
    }
}

#[test]
fn activity_action_parses_every_slug_it_emits() {
    for (variant, slug) in ACTIVITY_ACTIONS {
        let parsed: ActivityAction = slug.parse().expect("slug round-trips");
        assert_eq!(parsed, *variant);
    }
}

#[test]
fn activity_action_rejects_unknown_slug() {
    let err = "sideways"
        .parse::<ActivityAction>()
        .expect_err("unknown slug rejected");
    assert!(err.contains("sideways"), "{err}");
}

#[test]
fn activity_entity_display_matches_serde() {
    for (variant, slug) in ACTIVITY_ENTITIES {
        assert_eq!(variant.to_string(), *slug);
        assert_eq!(
            serde_json::to_value(variant).expect("serialize entity"),
            serde_json::Value::String((*slug).to_owned()),
        );
    }
}

#[test]
fn access_decision_display_and_try_from_agree() {
    for (variant, text) in [
        (AccessDecision::Allow, "allow"),
        (AccessDecision::Deny, "deny"),
    ] {
        assert_eq!(variant.to_string(), text);
        assert_eq!(
            AccessDecision::try_from(text.to_owned()).expect("parses"),
            variant
        );
        assert_eq!(
            serde_json::to_value(variant).expect("serialize decision"),
            serde_json::Value::String(text.to_owned()),
        );
    }
}

#[test]
fn access_decision_rejects_anything_else() {
    let err = AccessDecision::try_from("maybe".to_owned()).expect_err("rejected");
    assert!(err.contains("maybe"), "{err}");
}

#[test]
fn access_control_rule_round_trips_through_json() {
    let json = serde_json::json!({
        "id": "rule-1",
        "entity_type": "skill",
        "entity_id": "example_web_search",
        "rule_type": "role",
        "rule_value": "admin",
        "access": "deny",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-02T00:00:00Z",
    });
    let rule: AccessControlRule = serde_json::from_value(json).expect("deserialize rule");
    assert_eq!(rule.access, AccessDecision::Deny);
    assert_eq!(rule.rule_value, "admin");

    let back = serde_json::to_value(&rule).expect("serialize rule");
    assert_eq!(back["access"], "deny");
    assert_eq!(back["rule_type"], "role");
}

#[test]
fn hook_event_type_display_matches_as_str() {
    for variant in HOOK_EVENT_TYPES {
        assert_eq!(variant.to_string(), variant.as_str());
    }
}

#[test]
fn hook_event_type_serialises_as_its_pascal_case_name() {
    for variant in HOOK_EVENT_TYPES {
        assert_eq!(
            serde_json::to_value(variant).expect("serialize event type"),
            serde_json::Value::String(variant.as_str().to_owned()),
        );
        let parsed: HookEventType =
            serde_json::from_str(&format!("\"{}\"", variant.as_str())).expect("round-trips");
        assert_eq!(parsed, *variant);
    }
}

#[test]
fn hooks_file_omits_absent_description_and_headers() {
    let mut hooks = std::collections::HashMap::new();
    hooks.insert(
        HookEventType::PreToolUse,
        vec![MatcherGroup {
            matcher: "*".to_owned(),
            hooks: vec![HookHandler::Http(HttpHook {
                url: "https://example.test/hooks/govern".to_owned(),
                headers: None,
                timeout: Some(5),
            })],
        }],
    );
    let file = HooksFile {
        description: None,
        hooks,
    };

    let json = serde_json::to_value(&file).expect("serialize hooks file");
    assert!(json.get("description").is_none());
    let handler = &json["hooks"]["PreToolUse"][0]["hooks"][0];
    assert_eq!(handler["type"], "http");
    assert!(handler.get("headers").is_none());
    assert_eq!(handler["timeout"], 5);
}

fn payload(raw: serde_json::Value) -> (HookEventPayload, Vec<String>) {
    HookEventPayload::from_value(raw)
}

#[test]
fn every_known_hook_event_name_dispatches_to_its_variant() {
    for name in HOOK_EVENT_TYPES {
        let (parsed, warnings) = payload(serde_json::json!({
            "session_id": "sess-1",
            "cwd": "/repo",
            "hook_event_name": name.as_str(),
            "agent_id": "agent-1",
        }));
        assert_eq!(parsed.event_name(), name.as_str());
        assert!(
            !matches!(parsed.event, HookEvent::Unknown(_)),
            "{} fell through to Unknown: {warnings:?}",
            name.as_str()
        );
    }
}

#[test]
fn unknown_hook_event_name_is_preserved_and_warned_about() {
    let (parsed, warnings) = payload(serde_json::json!({
        "session_id": "sess-1",
        "cwd": "/repo",
        "hook_event_name": "Telepathy",
    }));
    assert_eq!(parsed.event_name(), "Telepathy");
    assert!(matches!(parsed.event, HookEvent::Unknown(_)));
    assert!(
        warnings.iter().any(|w| w.contains("Telepathy")),
        "{warnings:?}"
    );
}

#[test]
fn missing_common_fields_warn_rather_than_reject() {
    let (parsed, warnings) = payload(serde_json::json!({ "hook_event_name": "Stop" }));
    assert_eq!(parsed.session_id(), "");
    assert_eq!(parsed.cwd(), None);
    assert!(
        warnings.iter().any(|w| w.contains("session_id")),
        "{warnings:?}"
    );
    assert!(warnings.iter().any(|w| w.contains("cwd")), "{warnings:?}");
}

#[test]
fn an_empty_payload_resolves_to_the_unknown_event() {
    let (parsed, warnings) = payload(serde_json::json!({}));
    assert_eq!(parsed.event_name(), "unknown");
    assert!(
        warnings.iter().any(|w| w.contains("unknown")),
        "{warnings:?}"
    );
}

#[test]
fn subagent_events_warn_when_agent_id_is_absent() {
    for name in ["SubagentStart", "SubagentStop"] {
        let (_, warnings) = payload(serde_json::json!({
            "session_id": "sess-1",
            "cwd": "/repo",
            "hook_event_name": name,
        }));
        assert!(
            warnings.iter().any(|w| w.contains("agent_id")),
            "{name}: {warnings:?}"
        );
    }
}

#[test]
fn tool_accessors_read_the_variant_that_carries_the_tool() {
    let (pre, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "PreToolUse",
        "tool_name": "Bash", "tool_input": { "command": "ls" },
    }));
    assert_eq!(pre.tool_name(), Some("Bash"));
    assert_eq!(pre.tool_input().expect("input")["command"], "ls");

    let (failure, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "PostToolUseFailure",
        "tool_name": "Read", "tool_input": { "file_path": "/etc/hosts" }, "error": "denied",
    }));
    assert_eq!(failure.tool_name(), Some("Read"));

    let (stop, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "Stop",
    }));
    assert_eq!(stop.tool_name(), None);
    assert!(stop.tool_input().is_none());
}

#[test]
fn an_empty_tool_name_reads_as_absent() {
    let (parsed, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "PreToolUse", "tool_name": "",
    }));
    assert_eq!(parsed.tool_name(), None);
}

#[test]
fn prompt_and_model_come_only_from_their_own_events() {
    let (submit, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "UserPromptSubmit",
        "prompt": "deploy the thing",
    }));
    assert_eq!(submit.prompt(), Some("deploy the thing"));
    assert_eq!(submit.model(), None);

    let (start, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "SessionStart",
        "model": "claude-fable-5", "source": "startup",
    }));
    assert_eq!(start.model(), Some("claude-fable-5"));
    assert_eq!(start.prompt(), None);
}

#[test]
fn an_empty_prompt_or_model_reads_as_absent() {
    let (submit, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "UserPromptSubmit", "prompt": "",
    }));
    assert_eq!(submit.prompt(), None);

    let (start, _) = payload(serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "SessionStart", "model": "",
    }));
    assert_eq!(start.model(), None);
}

#[test]
fn the_raw_envelope_is_kept_whole() {
    let raw = serde_json::json!({
        "session_id": "s", "cwd": "/repo", "hook_event_name": "Stop",
        "vendor_specific": { "nested": [1, 2, 3] },
    });
    let (parsed, _) = payload(raw.clone());
    assert_eq!(parsed.raw, raw);
}

#[test]
fn tool_input_summary_keeps_known_keys_and_defaults_the_rest() {
    let summary = ToolInputSummary::of(&serde_json::json!({
        "command": "rm -rf /", "unrelated": 42,
    }));
    assert_eq!(summary.command.as_deref(), Some("rm -rf /"));
    assert_eq!(summary.file_path, None);
    assert_eq!(summary.pattern, None);

    let from_array = ToolInputSummary::of(&serde_json::json!([1, 2]));
    assert_eq!(from_array.command, None);
}

#[test]
fn dashboard_query_defaults_are_the_ranges_the_page_opens_on() {
    let query: DashboardQuery = serde_json::from_value(serde_json::json!({})).expect("defaults");
    assert_eq!(query.range, "7d");
    assert_eq!(query.traffic_range, "today");
    assert_eq!(query.content_range, "7d");
    assert_eq!(query.status, "");
    assert_eq!(query.tab, "");
}

#[test]
fn gateway_route_view_omits_an_absent_upstream_model() {
    let route = GatewayRouteView {
        id: "claude-abc".to_owned(),
        model_pattern: "claude-*".to_owned(),
        provider: "anthropic".to_owned(),
        ..Default::default()
    };
    let json = serde_json::to_value(&route).expect("serialize route");
    assert!(json.get("upstream_model").is_none());
    assert_eq!(json["extra_headers"], serde_json::json!({}));

    let back: GatewayRouteView = serde_json::from_value(json).expect("round-trips");
    assert_eq!(back.model_pattern, "claude-*");
    assert_eq!(back.upstream_model, None);
}
