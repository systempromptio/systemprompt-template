//! Shared entry point for the activity constructors.

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    // Why: `max` is a byte budget, and a description carries arbitrary user
    // text — slicing straight at it splits a multi-byte character and panics.
    let end = (0..=max)
        .rev()
        .find(|i| s.is_char_boundary(*i))
        .unwrap_or(0);
    let slice = &s[..end];
    slice
        .rfind(' ')
        .map_or_else(|| format!("{slice}..."), |pos| format!("{}...", &s[..pos]))
}
