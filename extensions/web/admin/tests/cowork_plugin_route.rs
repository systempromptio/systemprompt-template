use std::fs;

use systemprompt_web_admin::test_support::resolve_within;
use tempfile::TempDir;

#[test]
fn resolves_normal_relative_path() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(plugin_dir.join("agents"))?;
    fs::write(plugin_dir.join("agents/main.md"), b"hello")?;

    let resolved = resolve_within(&plugin_dir, "agents/main.md")
        .map_err(|e| anyhow::anyhow!("resolves: {e}"))?;
    assert_eq!(fs::read(&resolved)?, b"hello");
    Ok(())
}

#[test]
fn rejects_parent_traversal_component() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir)?;
    fs::write(tmp.path().join("secret.md"), b"top secret")?;

    let Err(err) = resolve_within(&plugin_dir, "../secret.md") else {
        anyhow::bail!("must reject parent traversal");
    };
    assert_eq!(err, "non-canonical component");
    Ok(())
}

#[test]
fn rejects_absolute_path() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir)?;

    let Err(err) = resolve_within(&plugin_dir, "/etc/passwd") else {
        anyhow::bail!("must reject absolute path");
    };
    assert_eq!(err, "non-canonical component");
    Ok(())
}

#[test]
fn rejects_empty_path() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir)?;

    let Err(err) = resolve_within(&plugin_dir, "") else {
        anyhow::bail!("must reject empty path");
    };
    assert_eq!(err, "empty path");
    Ok(())
}

#[test]
fn rejects_directory_target() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(plugin_dir.join("agents"))?;

    let Err(err) = resolve_within(&plugin_dir, "agents") else {
        anyhow::bail!("must reject directory target");
    };
    assert_eq!(err, "not a file");
    Ok(())
}
