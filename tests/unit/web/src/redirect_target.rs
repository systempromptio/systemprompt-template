//! A sign-in `redirect` is a path on this origin and nothing else: a form a
//! browser would resolve to another host is refused, never rewritten.

use systemprompt_web_shared::redirect::same_origin_path;

#[test]
fn a_path_on_this_origin_is_kept_with_its_query_and_fragment() {
    assert_eq!(
        same_origin_path("/admin/users").as_deref(),
        Some("/admin/users")
    );
    assert_eq!(
        same_origin_path("/admin/users?page=2#top").as_deref(),
        Some("/admin/users?page=2#top")
    );
}

#[test]
fn a_form_a_browser_resolves_off_site_is_refused() {
    for raw in [
        "//evil.example",
        "/\\evil.example",
        "/\\/evil.example",
        "\\evil.example",
        "https://evil.example/admin",
        "javascript:alert(1)",
        "/\tevil",
        "/\n/evil.example",
        " /admin",
        "admin/users",
        "",
    ] {
        assert_eq!(same_origin_path(raw), None, "{raw:?} must be refused");
    }
}

#[test]
fn dot_segments_cannot_climb_out_of_the_origin() {
    assert_eq!(same_origin_path("/admin/../../x").as_deref(), Some("/x"));
}
