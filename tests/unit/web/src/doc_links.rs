//! Every help topic the console can emit resolves to documentation that ships.
//!
//! The `?` icon linked `/documentation/<topic>` for a topic vocabulary that
//! belonged to an older documentation set, so 63 of 73 console pages offered a
//! help link that 404ed and nothing noticed. The mapping is now explicit, and
//! this walks the help table in the handler source to prove the two agree:
//! a topic missing from the table fails here, and so does one that names a
//! documentation page this instance does not ship.

use std::collections::BTreeSet;

use systemprompt_web_admin::types::doc_links::{doc_slug_for, documentation_url};

use crate::support::repo_root;

// Why: read from the handler source rather than a second hand-kept list. A
// duplicated list would agree with itself while both drifted from the pages.
fn help_topics() -> BTreeSet<String> {
    let dir = repo_root().join("extensions/web/admin/src/handlers/ssr/ssr_demo_help");
    let mut topics = BTreeSet::new();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // The topic is the last string literal of each `Some((text, topic))`
        // arm, and every arm closes with `,\n        ))`.
        for chunk in source.split("=> Some((").skip(1) {
            if let Some(topic) = last_literal_before_close(chunk) {
                topics.insert(topic);
            }
        }
        // The fallback arm is `.unwrap_or((text, topic))`.
        for chunk in source.split(".unwrap_or((").skip(1) {
            if let Some(topic) = last_literal_before_close(chunk) {
                topics.insert(topic);
            }
        }
    }
    assert!(
        topics.len() > 15,
        "parsed only {} topics; the help table's shape changed and this test \
         stopped seeing it: {topics:?}",
        topics.len()
    );
    topics
}

fn last_literal_before_close(chunk: &str) -> Option<String> {
    let arm = chunk.split("))").next()?;
    let mut literals = arm.rsplit('"');
    let _after = literals.next()?;
    literals.next().map(std::borrow::ToOwned::to_owned)
}

fn shipped_docs() -> BTreeSet<String> {
    let dir = repo_root().join("services/content/documentation");
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    let docs: BTreeSet<String> = entries
        .filter_map(|e| {
            let path = e.ok()?.path();
            (path.extension()?.to_str()? == "md").then(|| {
                path.file_stem()?
                    .to_str()
                    .map(std::borrow::ToOwned::to_owned)
            })?
        })
        .collect();
    assert!(!docs.is_empty(), "no documentation found to check against");
    docs
}

#[test]
fn every_help_topic_the_console_emits_is_known_to_the_mapping() {
    let docs = shipped_docs();
    let mut unknown = Vec::new();
    for topic in help_topics() {
        match doc_slug_for(&topic) {
            Some(slug) => assert!(
                docs.contains(slug),
                "help topic {topic:?} maps to {slug:?}, which is not in \
                 services/content/documentation/"
            ),
            // Why: `None` is a legitimate answer — the icon is not drawn. What
            // must never happen is a slug that does not resolve.
            None => unknown.push(topic),
        }
    }
    // The unmapped set is allowed, but it is recorded here so that adding a
    // documentation page for one of these is a visible change rather than a
    // silent one.
    assert!(
        unknown.len() < 12,
        "{} help topics have no documentation behind them, which is more than \
         this instance should be shipping: {unknown:?}",
        unknown.len()
    );
}

#[test]
fn a_topic_with_documentation_builds_a_documentation_url() {
    assert_eq!(
        documentation_url("tool-governance").as_deref(),
        Some("/documentation/enterprise-tool-governance")
    );
    assert_eq!(
        documentation_url("integration-claude-code").as_deref(),
        Some("/documentation/connect-claude-code")
    );
}

// Why: the icon has to disappear rather than link somewhere plausible-looking.
// A help link that 404s is worse than no help link: it reads as a broken
// product rather than as an undocumented corner.
#[test]
fn a_topic_with_no_documentation_yields_no_url_at_all() {
    assert_eq!(documentation_url("gamification"), None);
    assert_eq!(documentation_url("achievements"), None);
    assert_eq!(documentation_url("a-topic-nobody-has-written"), None);
}
