//! The sidebar's session list: order, archived and removed sessions, sessions
//! that change after the list was loaded, and children under their parent.

use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use systemprompt_engine::store::{DiffSummary, JsonMap, ProjectId, Session, SessionId};
use systemprompt_harness_views::state::session_list::{SessionList, SessionRow};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

fn session(title: &str, updated_at: i64, parent: Option<&SessionId>) -> Session {
    Session {
        id: SessionId::mint(&crate::IDS),
        project_id: ProjectId::mint(&crate::IDS),
        parent_id: parent.cloned(),
        directory: PathBuf::from("/work"),
        title: title.to_owned(),
        agent: None,
        model: None,
        permission: None,
        revert: None,
        summary: DiffSummary::default(),
        metadata: JsonMap::new(),
        created_at: updated_at,
        updated_at,
        archived_at: None,
    }
}

fn day(ms: i64) -> NaiveDate {
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|time| time.date_naive())
        .unwrap_or_default()
}

fn titles(list: &SessionList) -> Vec<String> {
    list.roots().iter().map(|s| s.title.clone()).collect()
}

fn listed(list: &SessionList) -> Vec<(String, u8)> {
    list.rows("", day(100 * DAY_MS), &HashSet::new(), day)
        .into_iter()
        .filter_map(|row| match row {
            SessionRow::Session { id, depth, .. } => {
                list.get(&id).map(|s| (s.title.clone(), depth))
            },
            SessionRow::Header(_) => None,
        })
        .collect()
}

#[test]
fn sessions_are_ordered_by_last_update_with_the_id_breaking_ties() {
    let mut list = SessionList::default();
    let first = session("first", 50, None);
    let second = session("second", 50, None);
    let newest = session("newest", 60, None);
    list.replace(vec![first, newest, second], Vec::new());
    assert_eq!(titles(&list), ["newest", "second", "first"]);
}

#[test]
fn archived_and_child_sessions_are_not_listed_as_roots() {
    let mut list = SessionList::default();
    let root = session("root", 10, None);
    let child = session("child", 20, Some(&root.id));
    let mut archived = session("archived", 30, None);
    archived.archived_at = Some(31);
    list.replace(vec![root, archived], vec![child]);
    assert_eq!(titles(&list), ["root"]);
}

#[test]
fn archiving_a_listed_session_takes_it_out_of_the_list() {
    let mut list = SessionList::default();
    let kept = session("kept", 10, None);
    let mut gone = session("gone", 20, None);
    list.replace(vec![kept, gone.clone()], Vec::new());
    gone.archived_at = Some(21);
    assert!(list.upsert(gone.clone()));
    assert!(!list.contains(&gone.id));
    assert_eq!(titles(&list), ["kept"]);
}

#[test]
fn a_deleted_session_leaves_the_list_and_its_children_with_it() {
    let mut list = SessionList::default();
    let root = session("root", 10, None);
    let child = session("child", 11, Some(&root.id));
    list.replace(vec![root.clone()], vec![child.clone()]);
    assert!(list.remove(&root.id));
    assert!(list.is_empty());
    assert!(!list.contains(&child.id));
    assert!(!list.remove(&root.id));
}

#[test]
fn a_session_created_after_the_load_is_listed_in_its_place() {
    let mut list = SessionList::default();
    list.replace(vec![session("old", 10, None)], Vec::new());
    assert!(list.upsert(session("new", 20, None)));
    assert_eq!(titles(&list), ["new", "old"]);
}

#[test]
fn a_session_updated_after_the_load_moves_to_the_top() {
    let mut list = SessionList::default();
    let mut quiet = session("quiet", 10, None);
    list.replace(vec![quiet.clone(), session("busy", 20, None)], Vec::new());
    quiet.updated_at = 30;
    assert!(list.upsert(quiet.clone()));
    assert_eq!(titles(&list), ["quiet", "busy"]);
    assert!(!list.upsert(quiet));
}

#[test]
fn child_sessions_are_listed_directly_after_their_parent() {
    let mut list = SessionList::default();
    let older = session("older", 100 * DAY_MS - 2, None);
    let parent = session("parent", 100 * DAY_MS - 1, None);
    let child = session("child", 100 * DAY_MS - 3, Some(&parent.id));
    let expanded: HashSet<SessionId> = [parent.id.clone()].into_iter().collect();
    list.replace(vec![older, parent], vec![child]);
    let rows: Vec<(String, u8)> = list
        .rows("", day(100 * DAY_MS), &expanded, day)
        .into_iter()
        .filter_map(|row| match row {
            SessionRow::Session { id, depth, .. } => {
                list.get(&id).map(|s| (s.title.clone(), depth))
            },
            SessionRow::Header(_) => None,
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("parent".to_owned(), 0),
            ("child".to_owned(), 1),
            ("older".to_owned(), 0)
        ]
    );
    assert_eq!(listed(&list).len(), 2);
}
