use std::path::Path;

use serde_json::Value;
use systemprompt_engine::config::{Action, Rule};
use systemprompt_harness_views::settings::permissions::jsonc::strip_jsonc;
use systemprompt_harness_views::settings::permissions::rules::{
    EditValue, RulesError, ShownRule, add_edit, group, parse_file, remove_edit, to_json,
};

use crate::TestResult;

fn home() -> &'static Path {
    Path::new("/home/user")
}

#[test]
fn comments_and_trailing_commas_are_stripped_outside_strings() -> TestResult {
    let text =
        "{\n  // a note\n  \"url\": \"http://x/*y*/\", /* gone */\n  \"list\": [1, 2,],\n}\n";
    let value: Value = serde_json::from_str(&strip_jsonc(text))?;
    assert_eq!(value["url"], "http://x/*y*/");
    assert_eq!(value["list"], serde_json::json!([1, 2]));
    Ok(())
}

#[test]
fn escaped_quotes_do_not_end_a_string() -> TestResult {
    let text = r#"{"a": "say \"hi\" // not a comment",}"#;
    let value: Value = serde_json::from_str(&strip_jsonc(text))?;
    assert_eq!(value["a"], "say \"hi\" // not a comment");
    Ok(())
}

#[test]
fn object_rules_keep_the_key_they_sit_under() -> TestResult {
    let file = parse_file(
        r#"{"permission": {"shell": {"*": "ask", "git *": "allow"}, "edit": "deny"}}"#,
        home(),
    )?;
    assert!(file.editable);
    assert_eq!(file.entries.len(), 3);
    assert_eq!(
        file.entries[1].rule,
        Rule::new("shell", "git *", Action::Allow)
    );
    assert_eq!(
        file.entries[1].key,
        Some(vec!["shell".to_owned(), "git *".to_owned()])
    );
    assert!(file.entries[2].shorthand);
    assert_eq!(file.entries[2].rule, Rule::new("edit", "*", Action::Deny));
    Ok(())
}

#[test]
fn list_rules_are_shown_but_not_editable() -> TestResult {
    let file = parse_file(
        r#"{"permission": [{"permission": "fetch", "pattern": "*", "action": "allow"}]}"#,
        home(),
    )?;
    assert!(!file.editable);
    assert_eq!(file.entries[0].key, None);
    assert_eq!(remove_edit(&file.entries[0]), None);
    Ok(())
}

#[test]
fn a_bad_action_is_refused() {
    assert!(matches!(
        parse_file(r#"{"permission": {"edit": "maybe"}}"#, home()),
        Err(RulesError::Shape { .. })
    ));
    assert!(matches!(parse_file("{", home()), Err(RulesError::Parse(_))));
}

#[test]
fn an_empty_file_is_editable_and_has_no_rules() -> TestResult {
    let file = parse_file("  // nothing yet\n", home())?;
    assert!(file.editable);
    assert_eq!(
        file.entries,
        Vec::<systemprompt_harness_views::settings::permissions::rules::Entry>::new()
    );
    Ok(())
}

#[test]
fn adding_beside_a_shorthand_keeps_it_first() -> TestResult {
    let file = parse_file(r#"{"permission": {"shell": "ask"}}"#, home())?;
    let edit = add_edit(&file, "shell", "git *", Action::Allow).ok_or("the rule is added")?;
    assert_eq!(edit.key, vec!["permission".to_owned(), "shell".to_owned()]);
    let expected = vec![
        ("*".to_owned(), Action::Ask),
        ("git *".to_owned(), Action::Allow),
    ];
    assert_eq!(edit.value, Some(EditValue::Patterns(expected)));
    assert_eq!(
        to_json(edit.value.as_ref().ok_or("the edit writes a value")?),
        serde_json::json!({"*": "ask", "git *": "allow"})
    );
    Ok(())
}

#[test]
fn adding_a_new_pattern_writes_one_key() -> TestResult {
    let file = parse_file("{}", home())?;
    let edit = add_edit(&file, " read ", " *.pem ", Action::Deny).ok_or("the rule is added")?;
    assert_eq!(
        edit.key,
        vec![
            "permission".to_owned(),
            "read".to_owned(),
            "*.pem".to_owned()
        ]
    );
    assert_eq!(edit.value, Some(EditValue::Action(Action::Deny)));
    assert_eq!(add_edit(&file, "read", "  ", Action::Deny), None);
    Ok(())
}

#[test]
fn removal_targets_the_full_key() -> TestResult {
    let file = parse_file(r#"{"permission": {"shell": {"rm *": "deny"}}}"#, home())?;
    let edit = remove_edit(&file.entries[0]).ok_or("the rule is removable")?;
    assert_eq!(
        edit.key,
        vec![
            "permission".to_owned(),
            "shell".to_owned(),
            "rm *".to_owned()
        ]
    );
    assert_eq!(edit.value, None);
    Ok(())
}

#[test]
fn groups_put_every_tool_first_and_keep_rule_order() {
    let shown = |permission: &str, pattern: &str, source| ShownRule {
        rule: Rule::new(permission, pattern, Action::Allow),
        source,
        removal: None,
    };
    let groups = group(vec![
        shown("shell", "a", 0),
        shown("*", "*", 0),
        shown("edit", "*", 1),
        shown("shell", "b", 2),
    ]);
    let names: Vec<&str> = groups
        .iter()
        .map(|group| group.permission.as_str())
        .collect();
    assert_eq!(names, vec!["*", "edit", "shell"]);
    let patterns: Vec<&str> = groups[2]
        .rules
        .iter()
        .map(|shown| shown.rule.pattern.as_str())
        .collect();
    assert_eq!(patterns, vec!["a", "b"]);
}
