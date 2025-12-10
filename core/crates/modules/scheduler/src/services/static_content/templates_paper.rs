use anyhow::Result;
use systemprompt_core_blog::models::paper::PaperMetadata;

use super::markdown::render_markdown;

pub fn calculate_read_time(html_content: &str) -> u32 {
    let text_count = html_content
        .replace(|c: char| c == '<' || c == '>', " ")
        .split_whitespace()
        .count();

    let minutes = (text_count as f32 / 200.0).ceil() as u32;
    minutes.max(1)
}

pub fn parse_paper_metadata(content: &str) -> Option<PaperMetadata> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1].trim();
    serde_yaml::from_str(frontmatter).ok()
}

pub fn generate_toc_html(paper_meta: &PaperMetadata) -> String {
    if paper_meta.sections.is_empty() {
        return String::new();
    }

    let items: Vec<String> = paper_meta
        .sections
        .iter()
        .map(|section| format!("<li><a href=\"#{}\">{}</a></li>", section.id, section.title))
        .collect();

    format!("<ul class=\"paper-toc__list\">{}</ul>", items.join("\n"))
}

fn split_content_by_sections(markdown_content: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_section_id = String::new();
    let mut current_content = String::new();

    for line in markdown_content.lines() {
        if line.starts_with("## ") {
            if !current_section_id.is_empty() || !current_content.trim().is_empty() {
                sections.push((
                    current_section_id.clone(),
                    current_content.trim().to_string(),
                ));
            }
            let title = line.trim_start_matches("## ").trim();
            current_section_id = title
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
                .replace("--", "-")
                .trim_matches('-')
                .to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_section_id.is_empty() || !current_content.trim().is_empty() {
        sections.push((current_section_id, current_content.trim().to_string()));
    }

    sections
}

fn extract_section_content(markdown_content: &str, section_id: &str) -> Option<String> {
    let start_marker = format!("<!-- SECTION_START: {} -->", section_id);
    let end_marker = format!("<!-- SECTION_END: {} -->", section_id);

    let start_pos = markdown_content.find(&start_marker)?;
    let end_pos = markdown_content.find(&end_marker)?;

    let content_start = start_pos + start_marker.len();
    if content_start >= end_pos {
        return None;
    }

    Some(markdown_content[content_start..end_pos].trim().to_string())
}

fn render_section_with_image(
    section_id: &str,
    section_title: &str,
    section_html: &str,
    image: Option<&str>,
    image_alt: Option<&str>,
    image_position: &str,
) -> String {
    let section_class = if image.is_some() {
        format!("paper-section paper-section--{image_position}")
    } else {
        "paper-section paper-section--no-image".to_string()
    };

    let image_html = if let Some(img_url) = image {
        let alt = image_alt.unwrap_or(section_title);
        format!(
            "<div class=\"paper-section__image\">\n      <img src=\"{}\" alt=\"{}\" \
             loading=\"lazy\" />\n    </div>",
            img_url, alt
        )
    } else {
        String::new()
    };

    format!(
        "<section id=\"{}\" class=\"{}\">\n    <div class=\"paper-section__text\">\n      \
         <h2>{}</h2>\n      {}\n    </div>\n    {}\n  </section>",
        section_id, section_class, section_title, section_html, image_html
    )
}

fn render_sections_with_markers(
    markdown_content: &str,
    paper_meta: &PaperMetadata,
) -> Result<Vec<String>> {
    let mut sections_html = Vec::new();

    for section in &paper_meta.sections {
        let section_markdown =
            extract_section_content(markdown_content, &section.id).unwrap_or_default();

        if section_markdown.is_empty() {
            continue;
        }

        let section_html = render_markdown(&section_markdown)?;

        let image = section.image.as_ref().map(|i| {
            if i.starts_with("http") || i.starts_with("/") {
                i.clone()
            } else {
                format!("/{i}")
            }
        });

        let html = render_section_with_image(
            &section.id,
            &section.title,
            &section_html,
            image.as_deref(),
            section.image_alt.as_deref(),
            &section.image_position,
        );

        sections_html.push(html);
    }

    Ok(sections_html)
}

fn render_sections_from_content(
    markdown_content: &str,
    paper_meta: &PaperMetadata,
    org_url: &str,
) -> Result<Vec<String>> {
    let content_sections = split_content_by_sections(markdown_content);
    let mut sections_html = Vec::new();

    for (position_index, (section_id, section_markdown)) in content_sections.iter().enumerate() {
        let section_html = render_markdown(section_markdown)?;

        let meta_section = paper_meta.sections.iter().find(|s| {
            s.id == *section_id
                || s.title
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
                    .replace("--", "-")
                    .trim_matches('-')
                    == *section_id
        });

        let (image, image_alt, image_position) = if let Some(meta) = meta_section {
            let img = meta.image.as_ref().map(|i| {
                if i.starts_with("http") {
                    i.clone()
                } else {
                    format!("{org_url}{i}")
                }
            });
            (img, meta.image_alt.clone(), meta.image_position.clone())
        } else {
            let pos = if position_index % 2 == 0 {
                "right"
            } else {
                "left"
            };
            (None, None, pos.to_string())
        };

        let section_title = meta_section
            .map(|m| m.title.clone())
            .unwrap_or_else(|| section_id.replace('-', " "));

        let html = render_section_with_image(
            section_id,
            &section_title,
            &section_html,
            image.as_deref(),
            image_alt.as_deref(),
            &image_position,
        );

        sections_html.push(html);
    }

    Ok(sections_html)
}

pub fn render_paper_sections_html(
    markdown_content: &str,
    paper_meta: &PaperMetadata,
    org_url: &str,
) -> Result<String> {
    let has_section_markers = markdown_content.contains("<!-- SECTION_START:");

    let sections_html = if has_section_markers {
        render_sections_with_markers(markdown_content, paper_meta)?
    } else {
        render_sections_from_content(markdown_content, paper_meta, org_url)?
    };

    Ok(sections_html.join("\n\n"))
}
