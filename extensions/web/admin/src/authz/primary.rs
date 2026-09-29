//! Ordering a membership list so the attribution key comes first.
//!
//! The gateway's quota resolver counts a window into the first value a
//! provider returns, and cost attribution counts by `user_scope_defaults`.
//! A person in several groups would otherwise be metered against the
//! alphabetically first one and costed against their primary — two answers
//! to one question. The authorization resolver matches on the whole set, so
//! the order is invisible to it.

// Why: promoted only when it is a current membership. A `manual` default
// survives every recomputation, so it can name a container the person no
// longer belongs to, and a quota must not count into one of those.
#[must_use]
pub fn lead_with_primary(primary: Option<&str>, mut ids: Vec<String>) -> Vec<String> {
    if let Some(index) = primary.and_then(|p| ids.iter().position(|id| id == p)) {
        let first = ids.remove(index);
        ids.insert(0, first);
    }
    ids
}
