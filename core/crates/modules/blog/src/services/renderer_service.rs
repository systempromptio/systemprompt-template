use anyhow::Result;

#[derive(Debug, Copy, Clone)]
pub struct RendererService;

impl RendererService {
    pub fn render_markdown_to_html(markdown: &str) -> Result<String> {
        use pulldown_cmark::{html, Parser};

        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(html_output)
    }

    pub fn render_with_template(
        title: &str,
        author: Option<&str>,
        published: Option<&str>,
        html_content: &str,
    ) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ border-bottom: 2px solid #333; }}
        .meta {{ color: #666; font-size: 0.9em; }}
        article {{ margin-top: 20px; line-height: 1.6; }}
        code {{ background-color: #f4f4f4; padding: 2px 6px; border-radius: 3px; }}
        pre {{ background-color: #f4f4f4; padding: 10px; border-radius: 3px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="meta">
        {}{}
    </div>
    <article>
        {}
    </article>
</body>
</html>"#,
            title,
            title,
            author.map(|a| format!("By {a} ")).unwrap_or_default(),
            published.map(|p| format!("· {p}")).unwrap_or_default(),
            html_content
        )
    }
}
