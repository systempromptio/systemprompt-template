pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let slice = &s[..max];
    slice.rfind(' ').map_or_else(|| format!("{slice}..."), |pos| format!("{}...", &s[..pos]))
}

pub(super) fn extract_project_name(path: &str) -> &str {
    path.rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
}
