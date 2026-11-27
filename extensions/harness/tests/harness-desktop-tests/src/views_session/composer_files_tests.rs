use std::fs;

use systemprompt_harness_views::composer::files::{FILE_LIMIT, walk};

#[test]
fn walks_files_and_skips_noise() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("src/nested")).unwrap();
    fs::create_dir_all(root.path().join(".git")).unwrap();
    fs::create_dir_all(root.path().join("node_modules/pkg")).unwrap();
    fs::create_dir_all(root.path().join("dist")).unwrap();
    fs::write(root.path().join(".gitignore"), "dist/\n*.log\n").unwrap();
    fs::write(root.path().join("src/main.rs"), "").unwrap();
    fs::write(root.path().join("src/nested/lib.rs"), "").unwrap();
    fs::write(root.path().join("src/nested/debug.log"), "").unwrap();
    fs::write(root.path().join(".git/config"), "").unwrap();
    fs::write(root.path().join("node_modules/pkg/index.js"), "").unwrap();
    fs::write(root.path().join("dist/out.js"), "").unwrap();
    let files = walk(root.path(), FILE_LIMIT);
    assert_eq!(
        files,
        vec![
            ".gitignore".to_owned(),
            "src/main.rs".to_owned(),
            "src/nested/lib.rs".to_owned()
        ]
    );
}

#[test]
fn the_walk_stops_at_the_limit() {
    let root = tempfile::tempdir().unwrap();
    for index in 0..5 {
        fs::write(root.path().join(format!("{index}.txt")), "").unwrap();
    }
    assert_eq!(walk(root.path(), 3).len(), 3);
}

#[test]
fn gitignore_patterns_and_negations_are_honoured() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("src/generated")).unwrap();
    fs::write(
        root.path().join(".gitignore"),
        "*.tmp\n!keep.tmp\nsrc/generated/\n",
    )
    .unwrap();
    fs::write(root.path().join("scratch.tmp"), "").unwrap();
    fs::write(root.path().join("keep.tmp"), "").unwrap();
    fs::write(root.path().join("src/generated/out.rs"), "").unwrap();
    fs::write(root.path().join("src/lib.rs"), "").unwrap();
    let files = walk(root.path(), FILE_LIMIT);
    assert_eq!(
        files,
        vec![
            ".gitignore".to_owned(),
            "keep.tmp".to_owned(),
            "src/lib.rs".to_owned()
        ]
    );
}
