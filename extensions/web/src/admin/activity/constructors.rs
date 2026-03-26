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

#[allow(dead_code)]
fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        path.to_string()
    }
}
