use serde_json::json;
use systemprompt::models::artifacts::{ImageArtifact, ToolResponse};

#[must_use]
pub fn input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "skill_id": {
                "type": "string",
                "description": "Must be 'blog_image_generation'"
            },
            "topic": {
                "type": "string",
                "description": "Blog post topic for image generation context"
            },
            "title": {
                "type": "string",
                "description": "Blog post title"
            },
            "summary": {
                "type": "string",
                "description": "Brief summary of the blog content"
            },
            "style_hints": {
                "type": "string",
                "description": "Optional style guidance (e.g., 'satirical', 'technical', 'professional')"
            },
            "aspect_ratio": {
                "type": "string",
                "enum": ["1:1", "16:9", "4:3"],
                "description": "Image aspect ratio. Default: 16:9 for blog headers"
            }
        },
        "required": ["skill_id", "topic", "title", "summary"]
    })
}

#[must_use]
pub fn output_schema() -> serde_json::Value {
    ToolResponse::<ImageArtifact>::schema()
}

pub fn build_image_prompt(
    skill_content: &str,
    topic: &str,
    title: &str,
    summary: &str,
    style_hints: Option<&str>,
) -> String {
    let style_section = style_hints
        .map(|s| format!("\nStyle guidance: {s}"))
        .unwrap_or_default();

    format!(
        "{skill_content}\n\n\
        Generate a featured image for a blog post.\n\n\
        **Topic:** {topic}\n\
        **Title:** {title}\n\
        **Summary:** {summary}{style_section}\n\n\
        Create a striking visual that captures the essence of this content. \
        CRITICAL: No text, words, letters, or numbers in the image."
    )
}
