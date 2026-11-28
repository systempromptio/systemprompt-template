//! The session list: recent sessions with no query, order kept through a
//! rename, a deleted session dropped, and archived sessions.

use std::collections::HashSet;

use chrono::NaiveDate;
use systemprompt_engine::store::SessionId;
use systemprompt_harness_views::state::session_list::{SessionList, SessionRow};

use crate::TestResult;
use crate::fixtures::session;

fn listed(list: &SessionList, query: &str) -> TestResult<Vec<SessionId>> {
    let today = NaiveDate::from_ymd_opt(2026, 11, 28).ok_or("a date")?;
    Ok(list
        .rows(query, today, &HashSet::new(), |_| today)
        .into_iter()
        .filter_map(|row| match row {
            SessionRow::Session { id, .. } => Some(id),
            SessionRow::Header(_) => None,
        })
        .collect())
}

fn three() -> (SessionList, [SessionId; 3]) {
    let ids = [(); 3].map(|()| SessionId::mint(&crate::IDS));
    let mut list = SessionList::default();
    list.replace(
        vec![
            session(&ids[0], "Old work", 10),
            session(&ids[1], "Middle work", 20),
            session(&ids[2], "New work", 30),
        ],
        Vec::new(),
    );
    (list, ids)
}

#[test]
fn with_no_query_the_recent_sessions_are_listed_newest_first() -> TestResult {
    let (list, [old, middle, new]) = three();
    assert_eq!(listed(&list, "")?, vec![new, middle, old]);
    Ok(())
}

#[test]
fn a_rename_keeps_the_session_in_its_place() -> TestResult {
    let (mut list, [old, middle, new]) = three();
    assert!(list.upsert(session(&middle, "Renamed", 20)));
    assert_eq!(listed(&list, "")?, vec![new, middle, old]);
    Ok(())
}

#[test]
fn a_session_deleted_while_listed_leaves_the_list() -> TestResult {
    let (mut list, [old, middle, new]) = three();
    assert!(list.remove(&middle));
    assert_eq!(listed(&list, "")?, vec![new, old]);
    Ok(())
}

#[test]
#[ignore = "parity ledger: an archived session leaves the session list and its search; the reference keeps it findable, dimmed"]
fn an_archived_session_can_still_be_found_by_search() -> TestResult {
    let (mut list, [_, middle, _]) = three();
    let mut archived = session(&middle, "Middle work", 20);
    archived.archived_at = Some(40);
    list.upsert(archived);
    assert_eq!(listed(&list, "middle")?, vec![middle]);
    Ok(())
}
