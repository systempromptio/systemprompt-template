//! Turning the ids a report cites back into console links. The model is
//! asked for `{kind, id}` evidence and never for a URL, so a report can only
//! point at pages this instance serves; the mapping lives beside the report
//! store so the job crate needs no knowledge of the console's routes.

use urlencoding::encode;

// Why: one href per evidence kind; an unknown kind links nowhere rather than
// guessing a page.
#[must_use]
pub fn evidence_href(kind: &str, id: &str) -> Option<String> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(match kind {
        "skill" => format!("/admin/analysis/skills/{}", encode(id)),
        "conversation" => format!("/admin/analysis/conversations/{}", encode(id)),
        "tool" => format!("/admin/tools?tool={}", encode(id)),
        "model" => format!("/admin/analysis/conversations?model={}", encode(id)),
        "person" => format!("/admin/analysis/conversations?user_id={}", encode(id)),
        "marketplace" => format!("/admin/analysis/versions/{}", encode(id)),
        _ => return None,
    })
}
