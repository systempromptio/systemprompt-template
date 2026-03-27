pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let slice = &s[..max];
    if let Some(pos) = slice.rfind(' ') {
        format!("{}...", &s[..pos])
    } else {
        format!("{slice}...")
    }
}

pub(super) fn extract_project_name(path: &str) -> &str {
    path.rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
}

