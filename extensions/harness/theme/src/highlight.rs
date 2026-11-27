//! The syntax palette handed to the toolkit's highlighter as its highlight
//! theme.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::sync::Arc;

use gpui_kit::component::highlighter::HighlightTheme;

use super::color::Color;
use super::tokens::Palette;

// Why: the toolkit's syntax style types cannot be built field by field outside
// it, but its highlight theme deserialises from the same JSON shape as its
// theme files.
#[must_use]
pub fn highlight_theme(id: &str, palette: &Palette) -> Option<Arc<HighlightTheme>> {
    let style = |color: Color| serde_json::json!({ "color": color.to_hex() });
    let mut syntax = serde_json::Map::new();
    for (name, color) in palette.syntax.named() {
        syntax.insert(name.to_owned(), style(color));
    }
    let s = &palette.syntax;
    let aliases = [
        ("comment.doc", s.comment),
        ("boolean", s.constant),
        ("constructor", s.type_name),
        ("enum", s.type_name),
        ("variant", s.constant),
        ("label", s.property),
        ("title", s.function),
        ("preproc", s.keyword),
        ("embedded", s.variable),
        ("variable.special", s.keyword),
        ("string.escape", s.constant),
        ("string.regex", s.string),
        ("string.special", s.string),
        ("string.special.symbol", s.constant),
        ("punctuation.bracket", s.punctuation),
        ("punctuation.delimiter", s.punctuation),
        ("punctuation.list_marker", s.punctuation),
        ("punctuation.special", s.operator),
        ("tag.doctype", s.tag),
        ("link_uri", palette.text.accent),
        ("link_text", palette.text.accent),
        ("text.literal", s.string),
    ];
    for (name, color) in aliases {
        syntax.insert(name.to_owned(), style(color));
    }
    syntax.insert(
        String::from("comment"),
        serde_json::json!({ "color": s.comment.to_hex(), "font_style": "italic" }),
    );
    let value = serde_json::json!({
        "name": format!("{id}-{}", if palette.dark { "dark" } else { "light" }),
        "appearance": if palette.dark { "dark" } else { "light" },
        "style": { "syntax": serde_json::Value::Object(syntax) },
    });
    match serde_json::from_value::<HighlightTheme>(value) {
        Ok(theme) => Some(Arc::new(theme)),
        Err(error) => {
            tracing::warn!(%error, theme = id, "syntax palette rejected; keeping the toolkit's highlight colours");
            None
        },
    }
}
