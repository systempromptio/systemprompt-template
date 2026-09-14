//! Bundle identity includes every dependency and exact asset metadata.

use std::collections::BTreeMap;
use systemprompt::identifiers::{ResourceRevisionId, SourceSnapshotId};
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, DependencyRef, RevisionBundle, RevisionFiles, RevisionManifest,
};

fn example() -> RevisionBundle {
    let root = ResourceRevisionId::generate();
    let dependency = ResourceRevisionId::generate();
    let files = RevisionFiles(BTreeMap::from([(
        "assets/reference.bin".to_owned(),
        AssetFile {
            bytes: vec![0, 255, 13, 10],
            media_type: "application/octet-stream".to_owned(),
            executable: true,
        },
    )]));
    let supporting =
        RevisionManifest::from_files(SourceSnapshotId::generate(), None, &files, BTreeMap::new())
            .unwrap();
    let root_files = RevisionFiles(BTreeMap::from([(
        "SKILL.md".to_owned(),
        AssetFile {
            bytes: b"Read references.\n".to_vec(),
            media_type: "text/markdown".to_owned(),
            executable: false,
        },
    )]));
    let manifest = RevisionManifest::from_files(
        SourceSnapshotId::generate(),
        None,
        &root_files,
        BTreeMap::from([(
            "references".to_owned(),
            DependencyRef {
                revision_id: dependency.clone(),
                digest: supporting.digest().unwrap(),
            },
        )]),
    )
    .unwrap();
    RevisionBundle {
        schema_version: 1,
        assembler_version: systemprompt::marketplace::managed::ASSEMBLER_VERSION.to_owned(),
        root: root.clone(),
        revisions: BTreeMap::from([(root, manifest), (dependency, supporting)]),
        assets: files
            .0
            .values()
            .chain(root_files.0.values())
            .map(|file| (AssetDigest::of(&file.bytes), file.bytes.clone()))
            .collect(),
    }
}

#[test]
fn bundle_round_trip_preserves_binary_modes_and_dependency_identity() {
    let bundle = example();
    let encoded = bundle.canonical_bytes().unwrap();
    let decoded: RevisionBundle = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.canonical_bytes().unwrap(), encoded);
    assert_eq!(decoded.digest().unwrap(), bundle.digest().unwrap());
    let dependency = &bundle.revisions[&bundle.root].dependencies["references"].revision_id;
    let files = decoded.revision_files(dependency).unwrap();
    assert_eq!(files.0["assets/reference.bin"].bytes, [0, 255, 13, 10]);
    assert!(files.0["assets/reference.bin"].executable);
    let mut changed = decoded;
    changed
        .revisions
        .get_mut(&bundle.root)
        .unwrap()
        .files
        .get_mut("SKILL.md")
        .unwrap()
        .executable = true;
    assert_ne!(changed.digest().unwrap(), bundle.digest().unwrap());
}

#[test]
fn bundle_refuses_modified_or_unreferenced_assets_and_incomplete_dependencies() {
    let bundle = example();
    let mut corrupted = bundle.clone();
    corrupted.assets.values_mut().next().unwrap().push(1);
    assert!(corrupted.verify().is_err());
    let mut extra = bundle.clone();
    extra
        .assets
        .insert(AssetDigest::of(b"unexpected"), b"unexpected".to_vec());
    assert!(extra.verify().is_err());
    let mut missing = bundle.clone();
    let dependency = missing.revisions[&missing.root].dependencies["references"]
        .revision_id
        .clone();
    missing.revisions.remove(&dependency);
    assert!(missing.verify().is_err());
    let mut wrong_pin = bundle.clone();
    wrong_pin
        .revisions
        .get_mut(&bundle.root)
        .unwrap()
        .dependencies
        .get_mut("references")
        .unwrap()
        .digest = AssetDigest::of(b"wrong");
    assert!(wrong_pin.verify().is_err());
    let mut unrelated = bundle.clone();
    unrelated.revisions.insert(
        ResourceRevisionId::generate(),
        bundle.revisions[&bundle.root].clone(),
    );
    assert!(unrelated.verify().is_err());
}

#[test]
fn bundle_refuses_unsupported_schemas_and_unsafe_paths() {
    let mut bundle = example();
    bundle.schema_version = 2;
    assert!(bundle.verify().is_err());
    bundle.schema_version = 1;
    let root = bundle.revisions.get_mut(&bundle.root).unwrap();
    root.schema_version = 2;
    assert!(bundle.verify().is_err());
    let root = bundle.revisions.get_mut(&bundle.root).unwrap();
    root.schema_version = 1;
    let file = root.files.remove("SKILL.md").unwrap();
    root.files.insert("../outside".to_owned(), file);
    assert!(bundle.verify().is_err());
}
