use std::fs;

use systemprompt_harness_views::composer::files::{FILE_LIMIT, walk};

use crate::TestResult;

#[test]
fn walks_files_and_skips_noise() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("src/nested"))?;
    fs::create_dir_all(root.path().join(".git"))?;
    fs::create_dir_all(root.path().join("node_modules/pkg"))?;
    fs::create_dir_all(root.path().join("dist"))?;
    fs::write(root.path().join(".gitignore"), "dist/\n*.log\n")?;
    fs::write(root.path().join("src/main.rs"), "")?;
    fs::write(root.path().join("src/nested/lib.rs"), "")?;
    fs::write(root.path().join("src/nested/debug.log"), "")?;
    fs::write(root.path().join(".git/config"), "")?;
    fs::write(root.path().join("node_modules/pkg/index.js"), "")?;
    fs::write(root.path().join("dist/out.js"), "")?;
    let files = walk(root.path(), FILE_LIMIT);
    assert_eq!(
        files,
        vec![
            ".gitignore".to_owned(),
            "src/main.rs".to_owned(),
            "src/nested/lib.rs".to_owned()
        ]
    );
    Ok(())
}

#[test]
fn the_walk_stops_at_the_limit() -> TestResult {
    let root = tempfile::tempdir()?;
    for index in 0..5 {
        fs::write(root.path().join(format!("{index}.txt")), "")?;
    }
    assert_eq!(walk(root.path(), 3).len(), 3);
    Ok(())
}

#[test]
fn gitignore_patterns_and_negations_are_honoured() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("src/generated"))?;
    fs::write(
        root.path().join(".gitignore"),
        "*.tmp\n!keep.tmp\nsrc/generated/\n",
    )?;
    fs::write(root.path().join("scratch.tmp"), "")?;
    fs::write(root.path().join("keep.tmp"), "")?;
    fs::write(root.path().join("src/generated/out.rs"), "")?;
    fs::write(root.path().join("src/lib.rs"), "")?;
    let files = walk(root.path(), FILE_LIMIT);
    assert_eq!(
        files,
        vec![
            ".gitignore".to_owned(),
            "keep.tmp".to_owned(),
            "src/lib.rs".to_owned()
        ]
    );
    Ok(())
}
