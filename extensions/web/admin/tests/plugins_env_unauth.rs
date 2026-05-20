//! Regression tests for the plugins-env principal-resolution invariant: the
//! handler must never synthesize a principal. When neither an authenticated
//! cookie session nor an explicit `user_id` query parameter is present,
//! resolution returns `None` and the handler returns 401.

use systemprompt_web_admin::test_support::resolve_principal;

#[test]
fn returns_none_when_neither_cookie_nor_query_present() {
    assert!(resolve_principal(None, None).is_none());
}

#[test]
fn prefers_cookie_user_id_over_query() -> anyhow::Result<()> {
    let resolved = resolve_principal(Some("alice"), Some("mallory"))
        .ok_or_else(|| anyhow::anyhow!("cookie principal must resolve"))?;
    assert_eq!(resolved.as_str(), "alice");
    Ok(())
}

#[test]
fn falls_back_to_query_user_id_when_cookie_absent() -> anyhow::Result<()> {
    let resolved = resolve_principal(None, Some("bob"))
        .ok_or_else(|| anyhow::anyhow!("query principal must resolve"))?;
    assert_eq!(resolved.as_str(), "bob");
    Ok(())
}

#[test]
fn uses_cookie_user_id_when_query_absent() -> anyhow::Result<()> {
    let resolved = resolve_principal(Some("carol"), None)
        .ok_or_else(|| anyhow::anyhow!("cookie principal must resolve"))?;
    assert_eq!(resolved.as_str(), "carol");
    Ok(())
}
