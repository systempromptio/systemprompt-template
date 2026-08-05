//! `html_escape` covers exactly four characters, and `&` must be replaced
//! first or the escapes it introduces would be double-escaped by the later
//! passes. The apostrophe is deliberately not escaped.

use systemprompt_web_shared::html_escape;

#[test]
fn ampersand_is_escaped_before_the_other_replacements() {
    // If `<` were replaced first, the `&` of `&lt;` would become `&amp;lt;`.
    assert_eq!(html_escape("a & b < c"), "a &amp; b &lt; c");
    assert_eq!(html_escape("&lt;"), "&amp;lt;");
}

#[test]
fn angle_brackets_and_quotes_are_escaped() {
    assert_eq!(
        html_escape("<script src=\"x\">"),
        "&lt;script src=&quot;x&quot;&gt;"
    );
}

#[test]
fn apostrophes_pass_through_unchanged() {
    assert_eq!(html_escape("it's fine"), "it's fine");
}

#[test]
fn text_without_special_characters_is_untouched() {
    assert_eq!(html_escape(""), "");
    assert_eq!(html_escape("plain text 123"), "plain text 123");
}
