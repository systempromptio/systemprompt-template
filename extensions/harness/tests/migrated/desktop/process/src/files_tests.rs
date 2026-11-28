//! The project file list offered for mentions: ignore-file directory
//! wildcards and a bounded scan.

use systemprompt_harness_views::composer::files::walk;

use crate::TestResult;

#[test]
fn directory_wildcards_in_the_ignore_file_hide_what_they_match() -> TestResult {
    let dir = tempfile::tempdir()?;
    let root = dir.path();
    for file in [
        "src/main.rs",
        "out/one/a.txt",
        "out/two/b.txt",
        "deep/x/cache/c.txt",
        "deep/x/keep.txt",
    ] {
        let path = root.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, "")?;
    }
    std::fs::write(root.join(".gitignore"), "out/*/\n**/cache/\n")?;
    let found = walk(root, 100);
    assert_eq!(found, vec![".gitignore", "deep/x/keep.txt", "src/main.rs"]);
    Ok(())
}

#[test]
fn a_scan_of_a_large_tree_stops_at_its_bound() -> TestResult {
    let dir = tempfile::tempdir()?;
    for index in 0..40 {
        let path = dir
            .path()
            .join(format!("d{}", index % 4))
            .join(format!("f{index}.txt"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, "")?;
    }
    assert_eq!(walk(dir.path(), 10).len(), 10);
    Ok(())
}
