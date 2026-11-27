use std::collections::HashSet;

use harness_drive_ui::model::{Browse, BrowseScope, Load, SortKey, folder_rows, sort_documents};
use sp_drive::view::DriveFailure;
use sp_protocol::wire::drive::DriveScope;

use crate::fixtures::{folder_id, id, item, node};

#[test]
fn a_folder_lists_by_folder_scope_and_search_is_trimmed() {
    let mut browse = Browse {
        query: "  q3 ".to_owned(),
        ..Browse::default()
    };
    let query = browse.list_query(None);
    assert_eq!(query.scope, DriveScope::Mine);
    assert_eq!(query.query.as_deref(), Some("q3"));
    browse.folder = Some(folder_id(1));
    browse.query = "   ".to_owned();
    let query = browse.list_query(Some("c".to_owned()));
    assert_eq!(query.scope, DriveScope::Folder);
    assert_eq!(query.folder, Some(folder_id(1)));
    assert_eq!(query.query, None);
    assert_eq!(query.cursor.as_deref(), Some("c"));
}

#[test]
fn scopes_map_to_their_wire_scope() {
    assert_eq!(
        BrowseScope::ALL.map(BrowseScope::wire),
        [DriveScope::Mine, DriveScope::Shared, DriveScope::Recent]
    );
}

#[test]
fn stale_pages_are_dropped_and_appended_pages_merge() {
    let mut browse = Browse::default();
    let old = browse.begin(false);
    let current = browse.begin(false);
    assert!(!browse.apply_page(old, vec![item(1, "a", "2026-01-01", None)], None));
    assert!(browse.documents.is_loading());
    assert!(browse.apply_page(
        current,
        vec![item(1, "a", "2026-01-01", None)],
        Some("next".to_owned())
    ));
    let more = browse.begin(true);
    assert!(browse.apply_page(
        more,
        vec![
            item(1, "a", "2026-01-01", None),
            item(2, "b", "2026-02-01", None)
        ],
        None
    ));
    let names: Vec<&str> = browse
        .documents
        .ready()
        .map(|items| items.iter().map(|i| i.name.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(names, vec!["b", "a"]);
    assert_eq!(browse.next_cursor, None);
    let failed = browse.begin(false);
    assert!(browse.fail_page(failed, DriveFailure::Unreachable));
    assert_eq!(browse.documents, Load::Failed(DriveFailure::Unreachable));
}

#[test]
fn a_reset_forgets_everything_and_drops_pages_in_flight() {
    let mut browse = Browse::default();
    let in_flight = browse.begin(false);
    browse.selected = Some(id(1));
    browse.folder = Some(folder_id(1));
    browse.reset();
    assert_eq!(browse.selected, None);
    assert_eq!(browse.folder, None);
    assert_eq!(browse.documents, Load::Idle);
    assert!(!browse.apply_page(in_flight, vec![item(1, "a", "1", None)], None));
}

#[test]
fn sorting_by_name_and_size() {
    let mut items = vec![
        item(1, "beta", "1", Some(5)),
        item(2, "Alpha", "2", None),
        item(3, "gamma", "3", Some(9)),
    ];
    sort_documents(&mut items, SortKey::Name);
    assert_eq!(items[0].name, "Alpha");
    sort_documents(&mut items, SortKey::Size);
    assert_eq!(items[0].name, "gamma");
    assert_eq!(items[2].name, "Alpha");
    assert_eq!(SortKey::from_key("size"), Some(SortKey::Size));
    assert_eq!(SortKey::from_key("other"), None);
}

#[test]
fn selection_steps_stay_inside_the_list() {
    let mut browse = Browse::default();
    let generation = browse.begin(false);
    browse.apply_page(
        generation,
        vec![item(1, "a", "3", None), item(2, "b", "2", None)],
        None,
    );
    assert_eq!(browse.step(true), Some(id(1)));
    browse.selected = Some(id(1));
    assert_eq!(browse.step(false), Some(id(1)));
    assert_eq!(browse.step(true), Some(id(2)));
    browse.selected = Some(id(2));
    assert_eq!(browse.step(true), Some(id(2)));
    assert_eq!(
        browse.item(&id(2)).map(|item| item.name.as_str()),
        Some("b")
    );
}

#[test]
fn folder_rows_follow_the_expanded_set() {
    let tree = vec![
        node(1, "Reports", vec![node(2, "2026", Vec::new())]),
        node(3, "Notes", Vec::new()),
    ];
    let mut browse = Browse::default();
    let rows = folder_rows(&tree, &browse.expanded);
    assert_eq!(rows.len(), 2);
    assert!(rows[0].has_children && !rows[0].expanded);
    browse.toggle_folder(&folder_id(1));
    let rows = folder_rows(&tree, &browse.expanded);
    assert_eq!(
        rows.iter()
            .map(|row| (row.name.as_str(), row.depth))
            .collect::<Vec<_>>(),
        vec![("Reports", 0), ("2026", 1), ("Notes", 0)]
    );
    browse.toggle_folder(&folder_id(1));
    assert_eq!(browse.expanded, HashSet::new());
}
