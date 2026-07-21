//! Shared entry point for the activity constructors.

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let slice = &s[..max];
    slice
        .rfind(' ')
        .map_or_else(|| format!("{slice}..."), |pos| format!("{}...", &s[..pos]))
}
