//! Same-origin redirect targets taken from a query string.

use url::Url;

const ORIGIN: &str = "http://same-origin.invalid/";

#[must_use]
pub fn same_origin_path(raw: &str) -> Option<String> {
    if !raw.starts_with('/')
        || raw.starts_with("//")
        || raw
            .chars()
            .any(|c| c == '\\' || c.is_control() || c.is_whitespace())
    {
        return None;
    }
    let base = Url::parse(ORIGIN).ok()?;
    let target = base.join(raw).ok()?;
    if target.origin() != base.origin() {
        return None;
    }
    let mut path = target.path().to_owned();
    if let Some(query) = target.query() {
        path.push('?');
        path.push_str(query);
    }
    if let Some(fragment) = target.fragment() {
        path.push('#');
        path.push_str(fragment);
    }
    Some(path)
}
