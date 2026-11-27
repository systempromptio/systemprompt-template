use std::path::{Path, PathBuf};

use systemprompt_engine::run::AttachmentInput;
use systemprompt_engine::store::SessionId;
use systemprompt_harness_views::composer::drafts::{Attachment, Draft, Drafts, add};

fn draft(text: &str) -> Draft {
    Draft {
        text: text.to_owned(),
        attachments: Vec::new(),
    }
}

#[test]
fn drafts_come_back_per_session() {
    let mut drafts = Drafts::default();
    let session = SessionId::mint(&crate::IDS);
    drafts.keep(Some(session.clone()), draft("half written"));
    drafts.keep(None, draft("new session"));
    assert_eq!(drafts.take(Some(&session)).text, "half written");
    assert!(drafts.take(Some(&session)).is_empty());
    assert_eq!(drafts.take(None).text, "new session");
}

#[test]
fn empty_drafts_are_dropped() {
    let mut drafts = Drafts::default();
    let session = SessionId::mint(&crate::IDS);
    drafts.keep(Some(session.clone()), draft("x"));
    drafts.keep(Some(session.clone()), draft("   "));
    assert!(drafts.take(Some(&session)).is_empty());
}

#[test]
fn attachments_are_not_added_twice() {
    let mut list = Vec::new();
    assert!(add(&mut list, [Attachment::file(Path::new("/tmp/a.png"))]));
    assert!(!add(&mut list, [Attachment::file(Path::new("/tmp/a.png"))]));
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].label, "a.png");
    assert_eq!(
        Attachment::mention("src/main.rs").input,
        AttachmentInput::Path(PathBuf::from("src/main.rs"))
    );
}
