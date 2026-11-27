//! What a permission request shows besides its title: the command, diff,
//! address or path it would act on, read from the metadata the tool attached to
//! the request.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    Command(String),
    Diff(String),
    Url(String),
    Path(String),
}

// JSON: the metadata a tool attaches to its permission request; its keys are
// the tool's own.
pub fn preview(metadata: &Value) -> Option<Preview> {
    let text = |key: &str| {
        metadata
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let files = || {
        metadata
            .get("files")
            .and_then(Value::as_array)
            .map(|files| {
                files
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|joined| !joined.is_empty())
    };
    text("diff")
        .map(Preview::Diff)
        .or_else(|| text("command").map(Preview::Command))
        .or_else(|| text("url").map(Preview::Url))
        .or_else(|| text("path").map(Preview::Path))
        .or_else(|| files().map(Preview::Path))
}
