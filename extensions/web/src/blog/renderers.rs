use serde_json::Value;

use crate::html_escape;

use super::types::{BlogPost, RelatedPost};

pub fn get_social_icon(platform_type: &str) -> &'static str {
    match platform_type {
        "github" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>"#
        }
        "twitter" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>"#
        }
        "linkedin" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"/></svg>"#
        }
        "email" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>"#
        }
        "share" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92-1.31-2.92-2.92-2.92z"/></svg>"#
        }
        _ => "",
    }
}

pub fn render_social_action_bar(_slug: &str, _title: &str, _org_url: &str) -> String {
    let social_links = [
        ("github", "https://github.com/systempromptio"),
        ("twitter", "https://twitter.com/systemprompt"),
        (
            "linkedin",
            "https://www.linkedin.com/company/systempromptio",
        ),
    ];

    let mut links_html = Vec::new();

    for (platform, href) in &social_links {
        let icon = get_social_icon(platform);
        if !icon.is_empty() {
            let label = match *platform {
                "github" => "GitHub",
                "twitter" => "Twitter",
                "linkedin" => "LinkedIn",
                _ => platform,
            };
            links_html.push(format!(
                r#"<a href="{href}" target="_blank" rel="noopener noreferrer" class="social-action-bar__link social-action-bar__link--{platform}" aria-label="Follow on {label}">{icon}</a>"#
            ));
        }
    }

    let share_icon = get_social_icon("share");
    links_html.push(format!(
        r#"<button type="button" class="social-action-bar__link social-action-bar__link--share" aria-label="Share this page" onclick="if(navigator.share){{navigator.share({{title:document.title,url:window.location.href}})}}else{{navigator.clipboard.writeText(window.location.href).then(()=>alert('Link copied!'))}}">{share_icon}</button>"#
    ));

    if links_html.is_empty() {
        return String::new();
    }

    format!(
        r#"<div class="social-action-bar">
  <span class="social-action-bar__label">Follow:</span>
  {}
</div>"#,
        links_html.join("\n  ")
    )
}

pub fn render_references(links: &Value) -> Option<String> {
    let arr = links.as_array()?;
    if arr.is_empty() {
        return None;
    }

    let cards: Vec<String> = arr
        .iter()
        .enumerate()
        .filter_map(|(i, link)| {
            let title = link.get("title")?.as_str()?;
            let url_str = link.get("url")?.as_str()?;
            let domain = url::Url::parse(url_str).ok()?.host_str()?.to_string();

            Some(format!(
                r#"<article class="reference-card">
  <span class="reference-card__number">[{}]</span>
  <a href="{}" class="reference-card__title" target="_blank" rel="noopener noreferrer">{}</a>
  <span class="reference-card__domain">{}</span>
</article>"#,
                i + 1,
                html_escape(url_str),
                html_escape(title),
                html_escape(&domain)
            ))
        })
        .collect();

    if cards.is_empty() {
        return None;
    }

    Some(format!(
        r#"<section class="references">
  <h2>References &amp; Sources</h2>
  <div class="references-grid">{}</div>
</section>"#,
        cards.join("\n")
    ))
}

pub fn render_related_posts(posts: &[RelatedPost]) -> Option<String> {
    if posts.is_empty() {
        return None;
    }

    let cards: Vec<String> = posts
        .iter()
        .map(|post| {
            format!(
                r#"<div class="related-post"><a href="/blog/{}">{}</a></div>"#,
                post.slug,
                html_escape(&post.title)
            )
        })
        .collect();

    Some(format!(
        r#"<section class="related-posts">
  <h3>Related Articles</h3>
  <div class="related-posts-list">
    {}
  </div>
</section>"#,
        cards.join("\n    ")
    ))
}

pub fn render_blog_cards(posts: &[BlogPost]) -> String {
    posts
        .iter()
        .map(|post| {
            let category = post.category.as_deref().unwrap_or("");
            let image = post
                .image
                .as_deref()
                .unwrap_or("/files/images/blog/placeholder.svg");
            let date = post.published_at.format("%B %d, %Y").to_string();
            let date_iso = post.published_at.format("%Y-%m-%d").to_string();

            format!(
                r#"<a href="/blog/{slug}" class="blog-card-link" data-category="{category}">
  <article class="blog-card content-card content-card--has-image" data-category="{category}">
    <div class="card-image">
      <img src="{image}" alt="{title}" loading="lazy" />
    </div>
    <div class="card-content">
      {category_badge}
      <h2 class="card-title">{title}</h2>
      <p class="card-description">{description}</p>
      <div class="meta">
        <time datetime="{date_iso}" class="meta-date">{date}</time>
      </div>
    </div>
  </article>
</a>"#,
                slug = post.slug,
                category = category,
                image = image,
                title = html_escape(&post.title),
                description = html_escape(&post.description),
                date = date,
                date_iso = date_iso,
                category_badge = if category.is_empty() {
                    String::new()
                } else {
                    format!(
                        r#"<span class="card-category card-category--{category}">{category}</span>"#
                    )
                },
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
