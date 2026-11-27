use systemprompt_engine::run::CommandInfo;
use systemprompt_harness_views::composer::popup::{
    Popup, PopupItem, PopupKind, Trigger, command_items, insert_mention, path_items, trigger,
};

fn command(name: &str) -> CommandInfo {
    CommandInfo {
        name: name.to_owned(),
        description: None,
        builtin: false,
    }
}

#[test]
fn a_leading_slash_opens_the_command_list() {
    assert_eq!(
        trigger("/com", 4),
        Some(Trigger::Slash {
            query: "com".to_owned()
        })
    );
    assert_eq!(trigger("/compact now", 12), None);
    assert_eq!(trigger("a /x", 4), None);
}

#[test]
fn an_at_sign_after_a_space_opens_the_file_list() {
    assert_eq!(
        trigger("look at @src/ma", 15),
        Some(Trigger::Mention {
            start: 8,
            query: "src/ma".to_owned()
        })
    );
    assert_eq!(trigger("mail me@example", 15), None);
    assert_eq!(
        trigger("@", 1),
        Some(Trigger::Mention {
            start: 0,
            query: String::new()
        })
    );
    assert_eq!(trigger("@done next", 10), None);
}

#[test]
fn cursors_inside_a_character_are_ignored() {
    assert_eq!(trigger("é", 1), None);
}

#[test]
fn commands_rank_prefix_matches_first() {
    let commands = vec![command("review"), command("compact"), command("new")];
    let items = command_items("e", &commands);
    let names: Vec<&str> = items.iter().map(|item| item.value.as_str()).collect();
    assert_eq!(names, vec!["new", "review"]);
}

#[test]
fn paths_prefer_file_name_matches() {
    let paths = vec![
        "src/main.rs".to_owned(),
        "docs/domain.md".to_owned(),
        "main/readme.md".to_owned(),
    ];
    let items = path_items("main", &paths);
    assert_eq!(items[0].value, "src/main.rs");
    assert_eq!(items[0].detail.as_deref(), Some("src"));
    assert_eq!(items.len(), 3);
}

#[test]
fn mentions_replace_the_typed_query() {
    let (text, caret) = insert_mention("see @ma please", 4, 7, "src/main.rs");
    assert_eq!(text, "see @src/main.rs  please");
    assert_eq!(caret, "see @src/main.rs ".len());
}

#[test]
fn popup_selection_wraps() {
    let item = PopupItem {
        value: "a".to_owned(),
        detail: None,
        builtin: false,
    };
    let mut popup = Popup {
        kind: PopupKind::Slash,
        items: vec![item.clone(), item],
        selected: 0,
    };
    popup.step(-1);
    assert_eq!(popup.selected, 1);
    popup.step(1);
    assert_eq!(popup.selected, 0);
    assert_eq!(popup.current().map(|item| item.value.as_str()), Some("a"));
}
