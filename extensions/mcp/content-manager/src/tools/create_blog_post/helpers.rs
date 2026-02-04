use serde_json::json;
use systemprompt::models::artifacts::{TextArtifact, ToolResponse};

#[must_use]
pub fn input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": {
                "type": "string",
                "description": "UUID from research_blog tool (optional for announcements, can be empty string)"
            },
            "skill_id": {
                "type": "string",
                "enum": ["blog_writing", "technical_content_writing", "announcement_writing", "guide_writing"],
                "description": "Writing skill to use: 'blog_writing' for narrative articles, 'technical_content_writing' for technical deep-dives, 'announcement_writing' for announcements, 'guide_writing' for step-by-step guides"
            },
            "slug": {
                "type": "string",
                "description": "URL slug for the blog post"
            },
            "description": {
                "type": "string",
                "description": "SEO meta description"
            },
            "keywords": {
                "type": "array",
                "items": {"type": "string"},
                "description": "SEO keywords"
            },
            "category": {
                "type": "string",
                "enum": ["announcement", "article", "guide"],
                "description": "Blog post category for filtering (default: 'article')"
            },
            "instructions": {
                "type": "string",
                "description": "Writing instructions and guidance"
            }
        },
        "required": ["skill_id", "slug", "description", "keywords", "instructions"]
    })
}

#[must_use]
pub fn output_schema() -> serde_json::Value {
    ToolResponse::<TextArtifact>::schema()
}

pub fn extract_title(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.trim().to_string();
        }
    }
    "Untitled".to_string()
}

pub fn build_user_prompt(
    summary: &str,
    sources: &[(String, String)],
    instructions: &str,
) -> String {
    let sources_section = if sources.is_empty() {
        String::new()
    } else {
        let sources_list: Vec<String> = sources
            .iter()
            .map(|(title, url)| format!("- [{title}]({url})"))
            .collect();
        format!("\n\n<sources>\n{}\n</sources>", sources_list.join("\n"))
    };

    format!(
        "<research>\n{summary}\n</research>{sources_section}\n\n<brief>\n{instructions}\n</brief>"
    )
}
