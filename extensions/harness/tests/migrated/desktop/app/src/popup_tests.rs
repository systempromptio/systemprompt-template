//! The composer's file mention list: what opens it, which entry is
//! highlighted after a new search, and how the keyboard moves the highlight.

use systemprompt_harness_views::composer::popup::{Popup, PopupKind, Trigger, path_items, trigger};

fn paths() -> Vec<String> {
    ["src/main.rs", "src/lib.rs", "docs/readme.md", "lib/util.rs"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_new_search_highlights_its_first_result() {
    let mut popup = Popup {
        kind: PopupKind::Mention { start: 0 },
        items: path_items("main", &paths()),
        selected: 0,
    };
    popup.step(1);
    popup = Popup {
        items: path_items("lib", &paths()),
        selected: 0,
        ..popup
    };
    assert_eq!(
        popup.current().map(|item| item.value.as_str()),
        Some("src/lib.rs")
    );
}

#[test]
fn confirming_never_picks_a_result_of_the_previous_query() {
    let items = path_items("readme", &paths());
    assert!(items.iter().all(|item| item.value.contains("readme")));
    assert_eq!(items.len(), 1);
}

#[test]
fn the_keyboard_highlight_moves_and_wraps_within_the_results() {
    let mut popup = Popup {
        kind: PopupKind::Mention { start: 0 },
        items: path_items("", &paths()),
        selected: 0,
    };
    let count = popup.items.len();
    popup.step(-1);
    assert_eq!(popup.selected, count - 1);
    popup.step(1);
    assert_eq!(popup.selected, 0);
}

#[test]
fn an_at_sign_inside_a_word_does_not_open_the_mention_list() {
    assert_eq!(trigger("mail me@home", 12), None);
    assert_eq!(
        trigger("look at @src", 12),
        Some(Trigger::Mention {
            start: 8,
            query: "src".to_owned()
        })
    );
}
