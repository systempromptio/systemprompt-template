use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use systemprompt_engine::store::{ProjectId, Session, SessionId};
use systemprompt_harness_views::state::session_list::{DayBucket, SessionList, SessionRow, bucket};

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
        summary: systemprompt_engine::store::DiffSummary::default(),
        metadata: systemprompt_engine::store::JsonMap::new(),
        created_at: updated_at,
        updated_at,
        archived_at: None,
    }
}

fn utc_day(ms: i64) -> NaiveDate {
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|time| time.date_naive())
        .unwrap_or_default()
}

fn today() -> NaiveDate {
    utc_day(100 * DAY_MS)
}

#[test]
fn days_fall_into_buckets() {
    let today = today();
    let ago = |days: u64| today - chrono::Days::new(days);
    assert_eq!(bucket(today, today), DayBucket::Today);
    assert_eq!(bucket(today, ago(1)), DayBucket::Yesterday);
    assert_eq!(bucket(today, ago(7)), DayBucket::PreviousWeek);
    assert_eq!(bucket(today, ago(30)), DayBucket::PreviousMonth);
    assert_eq!(bucket(today, ago(31)), DayBucket::Older);
    assert_eq!(
        bucket(today, today + chrono::Days::new(1)),
        DayBucket::Today
    );
}

#[test]
fn roots_are_newest_first_with_day_headers() {
    let mut list = SessionList::default();
    let old = session("old", 90 * DAY_MS, None);
    let recent = session("recent", 100 * DAY_MS, None);
    let yesterday = session("yesterday", 99 * DAY_MS, None);
    list.replace(vec![old, recent, yesterday], Vec::new());
    let rows = list.rows("", today(), &HashSet::new(), utc_day);
    let ids: Vec<_> = rows
        .iter()
        .map(|row| match row {
            SessionRow::Header(bucket) => format!("{bucket:?}"),
            SessionRow::Session { id, .. } => {
                list.get(id).map(|s| s.title.clone()).unwrap_or_default()
            },
        })
        .collect();
    assert_eq!(
        ids,
        [
            "Today",
            "recent",
            "Yesterday",
            "yesterday",
            "PreviousMonth",
            "old"
        ]
    );
}

#[test]
fn children_show_under_an_expanded_parent() {
    let mut list = SessionList::default();
    let parent = session("parent", 100 * DAY_MS, None);
    let child = session("child task", 100 * DAY_MS, Some(&parent.id));
    list.replace(vec![parent.clone()], vec![child.clone()]);
    let collapsed = list.rows("", today(), &HashSet::new(), utc_day);
    assert_eq!(collapsed.len(), 2);
    assert!(matches!(
        &collapsed[1],
        SessionRow::Session {
            has_children: true,
            expanded: false,
            ..
        }
    ));
    let expanded = list.rows("", today(), &HashSet::from([parent.id]), utc_day);
    assert_eq!(expanded.len(), 3);
    assert!(matches!(&expanded[2], SessionRow::Session { id, depth: 1, .. } if id == &child.id));
}

#[test]
fn search_matches_titles_and_opens_parents_of_matching_children() {
    let mut list = SessionList::default();
    let parent = session("Refactor", 100 * DAY_MS, None);
    let child = session("Write MIGRATION notes", 100 * DAY_MS, Some(&parent.id));
    let other = session("Unrelated", 100 * DAY_MS, None);
    list.replace(vec![parent.clone(), other], vec![child.clone()]);
    let rows = list.rows("migration", today(), &HashSet::new(), utc_day);
    assert_eq!(rows.len(), 3);
    assert!(matches!(&rows[1], SessionRow::Session { id, expanded: true, .. } if id == &parent.id));
    assert!(matches!(&rows[2], SessionRow::Session { id, .. } if id == &child.id));
    assert_eq!(
        list.rows("nothing", today(), &HashSet::new(), utc_day),
        Vec::<SessionRow>::new()
    );
}

#[test]
fn upserts_replace_reorder_and_drop_archived_sessions() {
    let mut list = SessionList::default();
    let first = session("first", 10, None);
    let second = session("second", 20, None);
    assert!(list.upsert(first.clone()));
    assert!(list.upsert(second.clone()));
    assert!(!list.upsert(second.clone()));
    assert_eq!(list.roots()[0].id, second.id);
    let mut touched = first.clone();
    touched.updated_at = 30;
    assert!(list.upsert(touched));
    assert_eq!(list.roots()[0].id, first.id);
    let mut archived = first.clone();
    archived.archived_at = Some(40);
    assert!(list.upsert(archived));
    assert!(!list.contains(&first.id));
}

#[test]
fn removing_a_parent_drops_its_children() {
    let mut list = SessionList::default();
    let parent = session("parent", 10, None);
    let child = session("child", 10, Some(&parent.id));
    list.replace(vec![parent.clone()], vec![child]);
    assert!(list.remove(&parent.id));
    assert_eq!(list.children(&parent.id), []);
    assert!(list.is_empty());
    assert!(!list.remove(&parent.id));
}
