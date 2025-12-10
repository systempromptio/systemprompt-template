use anyhow::Result;
use comrak::{markdown_to_html, ComrakOptions};

fn strip_first_h1(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut found_h1 = false;

    for line in lines {
        let trimmed = line.trim();
        if !found_h1 && trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            found_h1 = true;
            continue;
        }
        result.push(line);
    }

    result.join("\n")
}

pub fn render_markdown(content: &str) -> Result<String> {
    let mut options = ComrakOptions::default();

    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;

    options.render.unsafe_ = false;

    let content_without_h1 = strip_first_h1(content);
    let html = markdown_to_html(&content_without_h1, &options);
    Ok(html)
}

pub fn extract_frontmatter(content: &str) -> Option<(serde_yaml::Value, String)> {
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter_str = parts[1].trim();
    let body = parts[2].to_string();

    match serde_yaml::from_str(frontmatter_str) {
        Ok(yaml) => Some((yaml, body)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_basic() {
        let md = "# Hello\n\nThis is **bold**.";
        let html = render_markdown(md).unwrap();
        assert!(!html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_markdown_strips_first_h1() {
        let md = "# Title\n\nContent here\n\n## Subtitle";
        let html = render_markdown(md).unwrap();
        assert!(!html.contains("<h1>"));
        assert!(html.contains("<h2>Subtitle</h2>"));
        assert!(html.contains("Content here"));
    }

    #[test]
    fn test_render_markdown_preserves_h2() {
        let md = "## Subtitle\n\nContent";
        let html = render_markdown(md).unwrap();
        assert!(html.contains("<h2>Subtitle</h2>"));
    }

    #[test]
    fn test_render_markdown_list() {
        let md = "- Item 1\n- Item 2";
        let html = render_markdown(md).unwrap();
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_render_markdown_table() {
        let md = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let html = render_markdown(md).unwrap();
        assert!(html.contains("<table>"));
        assert!(html.contains("Header 1"));
        assert!(html.contains("Header 2"));
    }

    #[test]
    fn test_render_markdown_strikethrough() {
        let md = "~~strikethrough~~";
        let html = render_markdown(md).unwrap();
        assert!(html.contains("<del>strikethrough</del>"));
    }

    #[test]
    fn test_frontmatter_extraction() {
        let content = "---\ntitle: Test\nauthor: Edward\n---\n# Content";
        let (fm, body) = extract_frontmatter(content).unwrap();
        assert_eq!(fm["title"].as_str().unwrap(), "Test");
        assert_eq!(fm["author"].as_str().unwrap(), "Edward");
        assert!(body.contains("# Content"));
    }

    #[test]
    fn test_frontmatter_no_frontmatter() {
        let content = "# No frontmatter here";
        let result = extract_frontmatter(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_frontmatter_invalid_yaml() {
        let content = "---\ninvalid: yaml: content: here\n---\nBody";
        let result = extract_frontmatter(content);
        assert!(result.is_none());
    }
}
