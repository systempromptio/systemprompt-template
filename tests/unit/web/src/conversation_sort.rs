//! The sort contract of the conversation lists.
//!
//! The list binds its sort as a text parameter that a `CASE` in the
//! `ORDER BY` selects on, so the string a column maps to is load-bearing SQL,
//! not a label. A rename the query does not match does not fail to compile:
//! it silently falls through to the default ordering and the page quietly
//! stops honouring the header the user clicked.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::repositories::analytics::conversation_rows::{
    ConversationPage, ConversationSort,
};

#[test]
fn sort_columns_keep_the_keys_the_query_binds() {
    let cases = [
        (ConversationSort::Activity, "activity"),
        (ConversationSort::Turns, "turns"),
        (ConversationSort::Tokens, "tokens"),
        (ConversationSort::Cost, "cost"),
    ];
    for (column, expected) in cases {
        assert_eq!(column.as_str(), expected);
        assert_eq!(
            ConversationSort::parse_conversation_sort(Some(expected)),
            column,
            "the key a header emits must round-trip back to its column"
        );
    }
}

#[test]
fn an_unknown_sort_key_falls_back_to_activity() {
    for key in [None, Some(""), Some("nonsense"), Some("turnsX")] {
        assert_eq!(
            ConversationSort::parse_conversation_sort(key),
            ConversationSort::Activity
        );
    }
}

#[test]
fn the_default_page_is_newest_activity_first() {
    let page = ConversationPage::default();
    assert_eq!(page.sort, ConversationSort::Activity);
    assert!(page.descending);
    assert!(page.limit > 0);
    assert_eq!(page.offset, 0);
}
