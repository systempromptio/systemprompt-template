//! Source hashes are the operator's evidence that the declaration tree they
//! reviewed is the tree the sync page describes. A digest must cover every
//! file in a directory kind and change when either a file body changes or the
//! requested source path is absent.

use std::path::Path;

use systemprompt_web_admin::repositories::sync::tree_hash::subtree_hash;

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create fixture directory");
    std::fs::write(path, body).expect("write fixture file");
}

#[test]
fn a_file_source_digest_changes_with_its_declaration_body() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "gateway/policies.yaml", "policies: []\n");

    let first = subtree_hash(dir.path(), "gateway/policies.yaml", false).expect("file hash");
    assert_eq!(first.files, 1);

    write(
        dir.path(),
        "gateway/policies.yaml",
        "policies:\n  - name: quota\n",
    );
    let changed = subtree_hash(dir.path(), "gateway/policies.yaml", false).expect("file hash");
    assert_eq!(changed.files, 1);
    assert_ne!(
        changed.hash, first.hash,
        "source content changes its digest"
    );
}

#[test]
fn a_directory_source_digest_covers_each_nested_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "skills/a/SKILL.md", "first\n");
    write(dir.path(), "skills/b/config.yaml", "id: second\n");

    let first = subtree_hash(dir.path(), "skills", true).expect("directory hash");
    assert_eq!(first.files, 2);

    write(dir.path(), "skills/b/config.yaml", "id: changed\n");
    let changed = subtree_hash(dir.path(), "skills", true).expect("directory hash");
    assert_eq!(changed.files, 2);
    assert_ne!(
        changed.hash, first.hash,
        "nested content participates in the digest"
    );
}

#[test]
fn a_missing_source_has_no_digest_to_claim_as_active() {
    let dir = tempfile::tempdir().expect("tempdir");

    assert!(subtree_hash(dir.path(), "gateway/missing.yaml", false).is_none());
    assert!(subtree_hash(dir.path(), "skills", true).is_none());
}
