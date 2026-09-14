//! Portable, exact revision files and source provenance are validated at
//! ingress.

use std::collections::BTreeMap;
use systemprompt::identifiers::SourceSnapshotId;
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, RevisionFiles, RevisionManifest, SnapshotProvenance, SourceSpec,
};

fn files(path: &str) -> RevisionFiles {
    RevisionFiles(BTreeMap::from([(
        path.to_owned(),
        AssetFile {
            bytes: vec![0, 255, 128],
            media_type: "application/octet-stream".to_owned(),
            executable: false,
        },
    )]))
}

#[test]
fn revision_files_reject_traversal_and_excessive_content() {
    for path in [
        "../SKILL.md",
        "/SKILL.md",
        "a/../b",
        "a\\b",
        "C:/file",
        "a//b",
        "./x",
        "a\nfile",
    ] {
        assert!(files(path).validate().is_err(), "{path:?}");
    }
    assert!(files("assets/example.bin").validate().is_ok());
    let mut large = files("large.bin");
    large.0.get_mut("large.bin").expect("file").bytes = vec![0; 8 * 1024 * 1024 + 1];
    assert!(large.validate().is_err());
}

#[test]
fn manifest_hash_binds_permissions_paths_bytes_and_provenance() {
    let snapshot = SourceSnapshotId::generate();
    let original = files("SKILL.md");
    let first = RevisionManifest::from_files(snapshot.clone(), None, &original, BTreeMap::new())
        .expect("manifest");
    let same = RevisionManifest::from_files(snapshot.clone(), None, &original, BTreeMap::new())
        .expect("manifest");
    assert_eq!(first.digest().expect("hash"), same.digest().expect("hash"));
    let mut changed = original;
    changed.0.get_mut("SKILL.md").expect("file").executable = true;
    let second =
        RevisionManifest::from_files(snapshot, None, &changed, BTreeMap::new()).expect("manifest");
    assert_ne!(
        first.digest().expect("hash"),
        second.digest().expect("hash")
    );
    assert!(serde_json::from_str::<AssetDigest>("\"not-a-digest\"").is_err());
}

#[test]
fn git_snapshots_require_exact_commits_without_embedded_credentials() {
    let source = SourceSpec::Git {
        repository: "https://example.com/skills.git".to_owned(),
        reference: "main".to_owned(),
        subdirectory: None,
        credential_reference: None,
    };
    assert!(source.validate().is_ok());
    let mut snapshot = SnapshotProvenance {
        source_kind: "git".to_owned(),
        commit: Some("main".to_owned()),
        tree_digest: AssetDigest::of(b"tree"),
        importer_version: "v1".to_owned(),
    };
    assert!(snapshot.validate(&source).is_err());
    snapshot.commit = Some("a".repeat(40));
    assert!(snapshot.validate(&source).is_ok());
    let bad = SourceSpec::Git {
        repository: "https://token@example.com/skills.git".to_owned(),
        reference: "main".to_owned(),
        subdirectory: None,
        credential_reference: None,
    };
    assert!(bad.validate().is_err());
}

#[test]
fn authoring_capture_preserves_all_files_and_detects_changes() {
    use systemprompt::marketplace::managed::capture_skills;
    let dir = tempfile::tempdir().expect("source tree");
    let skill = dir.path().join("skills/test_skill");
    std::fs::create_dir_all(skill.join("assets")).expect("directories");
    std::fs::write(
        skill.join("config.yaml"),
        "id: test_skill\nname: Test\nenabled: true\nfile: SKILL.md\ndescription: Test skill\n",
    )
    .expect("config");
    std::fs::write(skill.join("SKILL.md"), "Read evidence.").expect("skill");
    std::fs::write(skill.join("assets/image.bin"), [0, 255, 128]).expect("binary");
    let ids = vec!["test_skill".to_owned()];
    let first = capture_skills(dir.path(), &ids).expect("capture");
    let same = capture_skills(dir.path(), &ids).expect("repeat");
    assert_eq!(first.tree_digest(), same.tree_digest());
    assert_eq!(
        first.skills()["test_skill"].0["assets/image.bin"].bytes,
        [0, 255, 128]
    );
    std::fs::write(skill.join("assets/image.bin"), [1, 255, 128]).expect("change");
    let changed = capture_skills(dir.path(), &ids).expect("changed capture");
    assert_ne!(first.tree_digest(), changed.tree_digest());
    assert!(capture_skills(dir.path(), &["../outside".to_owned()]).is_err());
}

#[cfg(unix)]
#[test]
fn authoring_capture_rejects_symlinks_instead_of_reading_outside_the_source() {
    use systemprompt::marketplace::managed::capture_skills;
    let dir = tempfile::tempdir().expect("source tree");
    let skill = dir.path().join("skills/test_skill");
    std::fs::create_dir_all(&skill).expect("directories");
    std::os::unix::fs::symlink("/etc/passwd", skill.join("SKILL.md")).expect("symlink");
    assert!(capture_skills(dir.path(), &["test_skill".to_owned()]).is_err());
}

#[test]
fn revision_diff_includes_metadata_changes_and_removed_files() {
    use systemprompt::marketplace::managed::diff_files;
    let snapshot = SourceSnapshotId::generate();
    let original = files("SKILL.md");
    let before = RevisionManifest::from_files(snapshot.clone(), None, &original, BTreeMap::new())
        .expect("manifest");
    let mut changed = original.clone();
    changed.0.get_mut("SKILL.md").expect("file").executable = true;
    let after = RevisionManifest::from_files(snapshot.clone(), None, &changed, BTreeMap::new())
        .expect("manifest");
    let diff = diff_files(&before, &after);
    assert_eq!(diff.len(), 1);
    assert_eq!(
        serde_json::to_value(diff[0].kind).expect("kind"),
        "modified"
    );
    let replaced =
        RevisionManifest::from_files(snapshot, None, &files("references/new.md"), BTreeMap::new())
            .expect("manifest");
    let diff = diff_files(&before, &replaced);
    assert_eq!(diff.len(), 2);
    assert_eq!(serde_json::to_value(diff[0].kind).expect("kind"), "removed");
    assert_eq!(serde_json::to_value(diff[1].kind).expect("kind"), "added");
}

#[test]
fn form_transport_preserves_original_line_endings_without_touching_indentation() {
    use systemprompt::marketplace::managed::normalize_form_text;
    assert_eq!(
        normalize_form_text("first\n  second\n", "first\r\n  edited\r\n").unwrap(),
        "first\n  edited\n"
    );
    assert_eq!(
        normalize_form_text("first\r\nsecond\r\n", "first\n  edited\n").unwrap(),
        "first\r\n  edited\r\n"
    );
    assert_eq!(
        normalize_form_text("first\rsecond\r", "first\r\nedited\r\n").unwrap(),
        "first\redited\r"
    );
    assert!(normalize_form_text("mixed\r\nendings\n", "edited\n").is_err());
    assert!(normalize_form_text("binary\0text", "edited").is_err());
}
