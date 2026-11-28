use chrono::{Local, TimeZone as _};
use systemprompt_drive::SyncPhase;
use systemprompt_harness::members::drive::DocumentKind;
use systemprompt_harness_views::drive::items::DriveFailure;
use systemprompt_harness_views::drive::model::BrowseScope;
use systemprompt_harness_views::drive::strings::{self as text, sync};

use crate::TestResult;

#[test]
fn sizes_and_times_read_well() -> TestResult {
    assert_eq!(text::size(512), "512 B");
    assert_eq!(text::size(1536), "1.5 KB");
    assert_eq!(text::size(5 * 1024 * 1024), "5.0 MB");
    let now = Local
        .with_ymd_and_hms(2026, 11, 25, 15, 0, 0)
        .single()
        .ok_or("a local time")?;
    assert_eq!(text::when("not a time", now), "not a time");
    let earlier = Local
        .with_ymd_and_hms(2025, 3, 4, 9, 30, 0)
        .single()
        .ok_or("a local time")?;
    assert_eq!(text::when(&earlier.to_rfc3339(), now), "04 Mar 2025");
    let today = Local
        .with_ymd_and_hms(2026, 11, 25, 9, 5, 0)
        .single()
        .ok_or("a local time")?;
    assert_eq!(text::when(&today.to_rfc3339(), now), "09:05");
    Ok(())
}

#[test]
fn every_kind_and_scope_has_its_own_label() {
    let kinds: Vec<&str> = DocumentKind::ALL.into_iter().map(text::kind).collect();
    let mut unique = kinds.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), kinds.len());
    assert_eq!(
        BrowseScope::ALL.map(text::scope),
        [text::SCOPE_MINE, text::SCOPE_SHARED, text::SCOPE_RECENT]
    );
}

#[test]
fn a_reference_names_the_document_and_its_id() {
    let reference = text::attach_reference("Q3 plan", "doc-1");
    assert!(reference.contains("\"Q3 plan\""));
    assert!(reference.contains("doc-1"));
    assert_eq!(text::uploading("a.md"), "Uploading a.md");
}

#[test]
fn sync_lines_say_what_is_waiting() {
    assert_eq!(sync::counts(3, 0), "3 file(s) synced");
    assert_eq!(sync::counts(3, 2), "3 file(s) synced, 2 waiting to upload");
    assert_ne!(sync::phase(SyncPhase::Off), sync::phase(SyncPhase::Error));
    assert_ne!(
        text::failure(DriveFailure::SignedOut),
        text::failure(DriveFailure::Failed)
    );
}
