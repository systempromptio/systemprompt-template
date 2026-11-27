//! The pure part of the Permissions page.
//!
//! It covers reading the rules a settings file writes (JSON with comments),
//! with the key each rule sits under so it can be removed; grouping rules by
//! permission in evaluation order; and the edit that adds a rule to the user's
//! settings without losing a shorthand rule already there.

use super::jsonc::strip_jsonc;
use std::path::PathBuf;

use serde_json::{Map, Value};
use systemprompt_engine::config::{Action, InvalidRule, Rule, Ruleset};

const PERMISSION_KEY: &str = "permission";
const ANY: &str = "*";

#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    #[error("the settings file is not valid JSON")]
    Parse(#[source] serde_json::Error),
    #[error("`{key}` is not an allow, ask or deny rule")]
    Shape { key: String },
    #[error("the permission rules are not valid")]
    Rules(#[source] InvalidRule),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub rule: Rule,
    pub key: Option<Vec<String>>,
    pub shorthand: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileRules {
    pub entries: Vec<Entry>,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditValue {
    Action(Action),
    Patterns(Vec<(String, Action)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleEdit {
    pub key: Vec<String>,
    pub value: Option<EditValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Source {
    Defaults(String),
    User(PathBuf),
    Project(PathBuf),
    Agent(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownRule {
    pub rule: Rule,
    pub source: usize,
    pub removal: Option<RuleEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub permission: String,
    pub rules: Vec<ShownRule>,
}

pub fn parse_file(text: &str, home: &std::path::Path) -> Result<FileRules, RulesError> {
    let stripped = strip_jsonc(text);
    if stripped.trim().is_empty() {
        return Ok(FileRules {
            entries: Vec::new(),
            editable: true,
        });
    }
    let root: Value = serde_json::from_str(&stripped).map_err(RulesError::Parse)?;
    match root.get(PERMISSION_KEY) {
        None | Some(Value::Null) => Ok(FileRules {
            entries: Vec::new(),
            editable: true,
        }),
        Some(Value::Object(map)) => Ok(FileRules {
            entries: walk(map)?,
            editable: true,
        }),
        Some(other) => {
            let rules = Ruleset::from_value(other, home).map_err(RulesError::Rules)?;
            let entries = rules
                .rules()
                .iter()
                .map(|rule| Entry {
                    rule: rule.clone(),
                    key: None,
                    shorthand: false,
                })
                .collect();
            Ok(FileRules {
                entries,
                editable: false,
            })
        },
    }
}

// JSON: the `permission` object of a settings file, read verbatim to keep each
// rule's key.
fn walk(map: &Map<String, Value>) -> Result<Vec<Entry>, RulesError> {
    let mut entries = Vec::new();
    for (permission, entry) in map {
        match entry {
            Value::String(action) => entries.push(Entry {
                rule: Rule::new(permission.as_str(), ANY, parse_action(permission, action)?),
                key: Some(vec![permission.clone()]),
                shorthand: true,
            }),
            Value::Object(patterns) => {
                for (pattern, action) in patterns {
                    let key = format!("{permission}.{pattern}");
                    let Value::String(action) = action else {
                        return Err(RulesError::Shape { key });
                    };
                    entries.push(Entry {
                        rule: Rule::new(
                            permission.as_str(),
                            pattern.as_str(),
                            parse_action(&key, action)?,
                        ),
                        key: Some(vec![permission.clone(), pattern.clone()]),
                        shorthand: false,
                    });
                }
            },
            _ => {
                return Err(RulesError::Shape {
                    key: permission.clone(),
                });
            },
        }
    }
    Ok(entries)
}

fn parse_action(key: &str, value: &str) -> Result<Action, RulesError> {
    Action::parse(value).ok_or_else(|| RulesError::Shape {
        key: key.to_owned(),
    })
}

pub fn add_edit(
    file: &FileRules,
    permission: &str,
    pattern: &str,
    action: Action,
) -> Option<RuleEdit> {
    let (permission, pattern) = (permission.trim(), pattern.trim());
    if !file.editable || permission.is_empty() || pattern.is_empty() {
        return None;
    }
    let shorthand = file
        .entries
        .iter()
        .find(|entry| entry.shorthand && entry.rule.permission == permission);
    let edit = match shorthand {
        Some(existing) if pattern != ANY => RuleEdit {
            key: vec![PERMISSION_KEY.to_owned(), permission.to_owned()],
            value: Some(EditValue::Patterns(vec![
                (ANY.to_owned(), existing.rule.action),
                (pattern.to_owned(), action),
            ])),
        },
        Some(_) => RuleEdit {
            key: vec![PERMISSION_KEY.to_owned(), permission.to_owned()],
            value: Some(EditValue::Action(action)),
        },
        None => RuleEdit {
            key: vec![
                PERMISSION_KEY.to_owned(),
                permission.to_owned(),
                pattern.to_owned(),
            ],
            value: Some(EditValue::Action(action)),
        },
    };
    Some(edit)
}

pub fn remove_edit(entry: &Entry) -> Option<RuleEdit> {
    let key = entry.key.as_ref()?;
    let mut full = vec![PERMISSION_KEY.to_owned()];
    full.extend(key.iter().cloned());
    Some(RuleEdit {
        key: full,
        value: None,
    })
}

// JSON: the global settings file stores a rule as an action string or an object
// of patterns to action strings.
pub fn to_json(value: &EditValue) -> Value {
    match value {
        EditValue::Action(action) => Value::String(action.as_str().to_owned()),
        EditValue::Patterns(patterns) => Value::Object(
            patterns
                .iter()
                .map(|(pattern, action)| {
                    (pattern.clone(), Value::String(action.as_str().to_owned()))
                })
                .collect(),
        ),
    }
}

pub fn group(rules: Vec<ShownRule>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for shown in rules {
        match groups
            .iter_mut()
            .find(|group| group.permission == shown.rule.permission)
        {
            Some(group) => group.rules.push(shown),
            None => groups.push(Group {
                permission: shown.rule.permission.clone(),
                rules: vec![shown],
            }),
        }
    }
    groups.sort_by(|left, right| {
        (left.permission != ANY, &left.permission)
            .cmp(&(right.permission != ANY, &right.permission))
    });
    groups
}
