#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

//! The parts of `/hooks/track`'s AI analysis that never reach a model: the
//! request context it stamps, the context block it assembles, the tool schema
//! it advertises, and the clamping every model response is put through before
//! it is stored.

use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin::test_support::{
    GeneratedSessionSummary, SessionAnalysis, build_full_context, build_request_context,
    session_analysis_schema, validate_analysis,
};

fn analysis(overrides: serde_json::Value) -> SessionAnalysis {
    let mut base = serde_json::json!({
        "title": "t", "description": "d", "goal_summary": "g",
        "outcomes": [], "tags": [], "goal_achieved": "yes",
        "quality_score": 3, "outcome": "o", "category": "feature",
    });
    let (serde_json::Value::Object(base_map), serde_json::Value::Object(extra)) =
        (&mut base, overrides)
    else {
        panic!("both sides must be JSON objects");
    };
    base_map.extend(extra);
    serde_json::from_value(base).expect("session analysis")
}

#[test]
fn a_quality_score_outside_one_to_five_is_clamped_into_range() {
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "quality_score": 99 }))).quality_score,
        5
    );
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "quality_score": -4 }))).quality_score,
        1
    );
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "quality_score": 3 }))).quality_score,
        3
    );
}

#[test]
fn a_goal_achieved_value_outside_the_enum_becomes_unknown() {
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "goal_achieved": "mostly" }))).goal_achieved,
        "unknown"
    );
    for allowed in ["yes", "partial", "no"] {
        assert_eq!(
            validate_analysis(analysis(serde_json::json!({ "goal_achieved": allowed })))
                .goal_achieved,
            allowed
        );
    }
}

#[test]
fn tags_outside_the_published_vocabulary_are_dropped_rather_than_stored() {
    let kept = validate_analysis(analysis(serde_json::json!({
        "tags": ["coding", "vibes", "debugging", "SHELL"],
    })));
    assert_eq!(kept.tags, ["coding", "debugging"]);
}

#[test]
fn an_unknown_or_missing_category_falls_back_to_other() {
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "category": "vibecoding" }))).category,
        Some("other".to_owned())
    );
    assert_eq!(
        validate_analysis(analysis(
            serde_json::json!({ "category": serde_json::Value::Null })
        ))
        .category,
        Some("other".to_owned())
    );
    assert_eq!(
        validate_analysis(analysis(serde_json::json!({ "category": "bugfix" }))).category,
        Some("bugfix".to_owned())
    );
}

#[test]
fn skill_scores_are_clamped_the_same_way_the_overall_score_is() {
    let kept = validate_analysis(analysis(serde_json::json!({
        "skill_scores": { "brand-voice": 9, "seo-guide": 0, "cli-usage": 4 },
    })));
    let scores = kept.skill_scores.expect("skill scores survive");
    assert_eq!(scores.get("brand-voice"), Some(&5));
    assert_eq!(scores.get("seo-guide"), Some(&1));
    assert_eq!(scores.get("cli-usage"), Some(&4));
}

#[test]
fn negative_efficiency_metrics_are_floored_at_zero() {
    let kept = validate_analysis(analysis(serde_json::json!({
        "efficiency_metrics": {
            "total_turns": -3, "duration_minutes": -1, "corrections_count": -2,
            "avg_turns_per_goal": -0.5, "unnecessary_loops": -7,
        },
    })));
    let eff = kept.efficiency_metrics.expect("metrics survive");
    assert_eq!(eff.total_turns, 0);
    assert_eq!(eff.duration_minutes, 0);
    assert_eq!(eff.corrections_count, 0);
    assert!((eff.avg_turns_per_goal - 0.0).abs() < f32::EPSILON);
    assert_eq!(eff.unnecessary_loops, 0);
}

#[test]
fn a_best_practice_score_outside_the_enum_becomes_not_applicable() {
    let kept = validate_analysis(analysis(serde_json::json!({
        "best_practices_checklist": [
            { "practice": "clear instructions", "score": "excellent", "note": "n" },
            { "practice": "sufficient context", "score": "partial", "note": "n" },
        ],
    })));
    let checklist = kept.best_practices_checklist.expect("checklist survives");
    assert_eq!(checklist[0].score, "n/a");
    assert_eq!(checklist[1].score, "partial");
}

#[test]
fn validation_leaves_an_already_conforming_analysis_untouched() {
    let kept = validate_analysis(analysis(serde_json::json!({
        "quality_score": 4, "tags": ["testing"], "goal_achieved": "partial",
        "category": "testing",
    })));
    assert_eq!(kept.quality_score, 4);
    assert_eq!(kept.tags, ["testing"]);
    assert_eq!(kept.goal_achieved, "partial");
    assert_eq!(kept.category, Some("testing".to_owned()));
}

#[test]
fn the_summary_is_composed_from_the_goal_and_its_outcome_bullets() {
    let composed = analysis(serde_json::json!({
        "goal_summary": "Ship the filter", "outcomes": ["Filter shipped", "Tests added"],
    }))
    .composed_summary();
    assert_eq!(
        composed,
        "Ship the filter\n\n- Filter shipped\n- Tests added"
    );
}

#[test]
fn a_summary_with_neither_goal_nor_outcomes_is_empty_rather_than_punctuation() {
    let composed =
        analysis(serde_json::json!({ "goal_summary": "", "outcomes": [] })).composed_summary();
    assert!(composed.is_empty());

    let only_outcomes =
        analysis(serde_json::json!({ "goal_summary": "", "outcomes": ["a"] })).composed_summary();
    assert_eq!(only_outcomes, "- a");
}

#[test]
fn the_advertised_schema_requires_every_field_the_dashboard_reads() {
    let schema = session_analysis_schema();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required list")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    for field in [
        "title",
        "description",
        "goal_summary",
        "category",
        "goal_outcome_map",
        "outcomes",
        "tags",
        "goal_achieved",
        "quality_score",
        "outcome",
        "efficiency_metrics",
        "best_practices_checklist",
    ] {
        assert!(required.contains(&field), "{field} is not required");
        assert!(
            schema["properties"].get(field).is_some(),
            "{field} is required but never described"
        );
    }
}

#[test]
fn every_enum_in_the_schema_matches_what_validation_will_accept() {
    let schema = session_analysis_schema();
    let enum_of = |path: &str| -> Vec<String> {
        schema["properties"][path]["enum"]
            .as_array()
            .expect("enum list")
            .iter()
            .map(|v| v.as_str().expect("string").to_owned())
            .collect()
    };

    let tags: Vec<String> = schema["properties"]["tags"]["items"]["enum"]
        .as_array()
        .expect("tag enum")
        .iter()
        .map(|v| v.as_str().expect("string").to_owned())
        .collect();
    let surviving = validate_analysis(analysis(serde_json::json!({ "tags": tags.clone() }))).tags;
    assert_eq!(
        surviving, tags,
        "validation drops a tag the schema tells the model to use"
    );

    let categories = enum_of("category");
    for category in categories {
        let kept = validate_analysis(analysis(serde_json::json!({ "category": &category })));
        assert_eq!(
            kept.category.as_deref(),
            Some(category.as_str()),
            "validation rewrote a category the schema tells the model to use"
        );
    }

    assert_eq!(enum_of("goal_achieved"), ["yes", "partial", "no"]);
}

fn summary(text: &str, tags: &str) -> GeneratedSessionSummary {
    GeneratedSessionSummary {
        summary: text.to_owned(),
        tags: tags.to_owned(),
    }
}

#[test]
fn without_an_events_summary_the_context_is_the_analysis_block_verbatim() {
    assert_eq!(
        build_full_context("SESSION: 3 prompts", None),
        "SESSION: 3 prompts"
    );
    assert_eq!(build_full_context("", None), "");
}

#[test]
fn an_events_summary_is_appended_as_an_activity_line_with_its_tags() {
    let full = build_full_context(
        "SESSION: 3 prompts",
        Some(&summary("2 tool calls.", "coding")),
    );
    assert_eq!(
        full,
        "SESSION: 3 prompts\nActivity: 2 tool calls.\nTags: coding"
    );
}

#[test]
fn an_events_summary_with_no_tags_omits_the_tag_line_entirely() {
    let full = build_full_context("SESSION: 3 prompts", Some(&summary("2 tool calls.", "")));
    assert_eq!(full, "SESSION: 3 prompts\nActivity: 2 tool calls.");
}

#[test]
fn with_no_analysis_block_the_events_summary_stands_alone_without_a_label() {
    let full = build_full_context("", Some(&summary("2 tool calls.", "coding,shell")));
    assert_eq!(full, "2 tool calls.\nTags: coding,shell");
}

#[test]
fn the_request_context_carries_the_session_and_the_hook_summary_agent() {
    let ctx = build_request_context(
        &UserId::new("11111111-1111-4111-8111-111111111111"),
        &SessionId::new("sess-1"),
        "jwt-token",
    );
    assert_eq!(ctx.request.session_id.as_str(), "sess-1");
    assert_eq!(ctx.execution.agent_name.as_str(), "hook-summary");
    assert!(!ctx.execution.trace_id.as_str().is_empty());
    assert_eq!(ctx.auth.auth_token.as_str(), "jwt-token");
    assert_eq!(
        ctx.user.as_ref().map(|u| u.id.to_string()),
        Some("11111111-1111-4111-8111-111111111111".to_owned())
    );
}

#[test]
fn every_summary_of_one_session_shares_a_context_and_two_sessions_never_collide() {
    let ctx_of = |s: &str| {
        build_request_context(
            &UserId::new("11111111-1111-4111-8111-111111111111"),
            &SessionId::new(s),
            "jwt",
        )
        .execution
        .context_id
        .as_str()
        .to_owned()
    };

    assert_eq!(
        ctx_of("sess-1"),
        ctx_of("sess-1"),
        "a re-analysed session must land in the context its earlier summaries used"
    );
    assert_ne!(ctx_of("sess-1"), ctx_of("sess-2"));
    assert_eq!(
        ctx_of("sess-1"),
        "0cb9c4c8-6b84-5c1a-b6e6-a148690fa761",
        "the session-context namespace is fixed forever; a different value re-homes every \
         historical summary"
    );
}

#[test]
fn a_non_uuid_user_id_yields_the_nil_uuid_rather_than_failing_the_analysis() {
    let ctx = build_request_context(&UserId::new("legacy-user"), &SessionId::new("s"), "jwt");
    let user = ctx.user.as_ref().expect("user is attached");
    assert!(
        user.id.is_nil(),
        "an unparseable id must degrade, not panic"
    );
    assert_eq!(user.username, "legacy-user");
}
