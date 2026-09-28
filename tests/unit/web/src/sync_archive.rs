//! The configuration archive: an upload is refused under every rule a
//! hostile zip would break, a good one is classified into plane files and
//! "served from code" entries, and the manifest round-trips.

use std::io::Write;

use chrono::Utc;
use systemprompt_web_admin::repositories::sync::archive::manifest::{
    ArchiveManifest, MANIFEST_FORMAT, ManifestPlane, parse_manifest, render_manifest,
};
use systemprompt_web_admin::repositories::sync::archive::read::{classify, unpack_zip};
use systemprompt_web_admin::repositories::sync::archive::{MANIFEST_FILE, MAX_ENTRY_BYTES};
use systemprompt_web_admin::repositories::sync::registry::planes;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, body) in entries {
        w.start_file(*name, options).expect("entry");
        w.write_all(body).expect("body");
    }
    w.finish().expect("finish").into_inner()
}

fn refusal(entries: &[(&str, &[u8])]) -> String {
    match unpack_zip(&zip_of(entries)) {
        Ok(_) => panic!("archive was accepted"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn a_good_archive_unpacks_and_classifies() {
    let manifest = ArchiveManifest {
        format: MANIFEST_FORMAT,
        release: "0.54.0".to_owned(),
        exported_at: Utc::now(),
        exported_by: "ed".to_owned(),
        base_tree_hash: Some("abc".to_owned()),
        composed_hash: None,
        planes: vec![ManifestPlane {
            id: "gateway_policies".to_owned(),
            file: "gateway/policies.yaml".to_owned(),
            declared_hash: None,
            applied_hash: None,
            applied_mode: None,
            applied_at: None,
            row_count: 0,
        }],
    };
    let rendered = render_manifest(&manifest).expect("renders");
    let bytes = zip_of(&[
        ("services/gateway/policies.yaml", b"policies: []\n"),
        ("services/skills/who_am_i/config.yaml", b"id: who_am_i\n"),
        ("services/", b""),
        (MANIFEST_FILE, rendered.as_bytes()),
    ]);
    let unpacked = unpack_zip(&bytes).expect("accepted");
    assert_eq!(unpacked.entries.len(), 2);
    let m = unpacked.manifest.as_ref().expect("manifest parsed");
    assert_eq!(m.release, "0.54.0");
    assert_eq!(m.planes[0].id, "gateway_policies");

    let staged = classify(unpacked, &planes(), "ed").expect("classified");
    assert_eq!(
        staged.planes.get("gateway_policies").map(String::as_str),
        Some("policies: []\n")
    );
    assert_eq!(staged.other.len(), 1);
    assert_eq!(staged.other[0].kind_id, Some("skills"));
    assert_eq!(staged.other[0].hash.len(), 12);
}

#[test]
fn a_missing_manifest_is_not_an_error() {
    let unpacked = unpack_zip(&zip_of(&[(
        "services/gateway/policies.yaml",
        b"policies: []\n",
    )]))
    .expect("accepted");
    assert!(unpacked.manifest.is_none());
    assert!(unpacked.manifest_error.is_none());
}

#[test]
fn a_bad_manifest_is_a_warning_not_a_refusal() {
    let unpacked = unpack_zip(&zip_of(&[
        ("services/gateway/policies.yaml", b"policies: []\n"),
        (MANIFEST_FILE, b"format: [not, a, manifest\n"),
    ]))
    .expect("accepted");
    assert!(unpacked.manifest.is_none());
    assert!(unpacked.manifest_error.is_some());
}

#[test]
fn parent_and_absolute_paths_are_refused() {
    assert!(refusal(&[("../x.yaml", b"x")]).contains("parent"));
    assert!(refusal(&[("services/../../x.yaml", b"x")]).contains("parent"));
    assert!(refusal(&[("/etc/passwd", b"x")]).contains("parent"));
}

#[test]
fn entries_outside_services_or_in_an_unknown_dir_are_refused() {
    assert!(refusal(&[("README.md", b"x")]).contains("outside services/"));
    assert!(refusal(&[("services/secrets/key.pem", b"x")]).contains("not a services directory"));
}

#[test]
fn backslashes_are_refused() {
    assert!(refusal(&[("services\\gateway\\policies.yaml", b"x")]).contains("backslash"));
}

#[test]
fn an_oversized_entry_is_refused_by_what_it_delivers() {
    let big = vec![b'a'; usize::try_from(MAX_ENTRY_BYTES).expect("fits") + 1];
    assert!(refusal(&[("services/gateway/policies.yaml", &big)]).contains("larger"));
}

#[test]
fn a_plane_file_that_is_not_text_is_refused_at_classification() {
    let unpacked = unpack_zip(&zip_of(&[(
        "services/gateway/policies.yaml",
        &[0xff, 0xfe, 0x00],
    )]))
    .expect("bytes unpack");
    let err = classify(unpacked, &planes(), "ed").expect_err("not text");
    assert!(err.to_string().contains("not UTF-8"), "{err}");
}

#[test]
fn not_a_zip_is_refused() {
    assert!(unpack_zip(b"policies: []").is_err());
}

#[test]
fn the_manifest_round_trips() {
    let manifest = ArchiveManifest {
        format: MANIFEST_FORMAT,
        release: "1.2.3".to_owned(),
        exported_at: Utc::now(),
        exported_by: "someone".to_owned(),
        base_tree_hash: None,
        composed_hash: Some("deadbeef".to_owned()),
        planes: Vec::new(),
    };
    let text = render_manifest(&manifest).expect("renders");
    assert!(text.starts_with("# Configuration archive"));
    let back = parse_manifest(&text).expect("parses");
    assert_eq!(back.release, "1.2.3");
    assert_eq!(back.composed_hash.as_deref(), Some("deadbeef"));
}
