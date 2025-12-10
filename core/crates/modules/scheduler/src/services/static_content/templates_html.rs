use serde_json::Value;
use systemprompt_core_blog::services::LinkGenerationService;
use systemprompt_core_database::DbPool;

use super::cards::{generate_related_card, CardData};

fn format_short_date(date_str: &str) -> String {
    if date_str.is_empty() {
        return String::new();
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%b %d, %Y").to_string()
    } else {
        date_str.to_string()
    }
}

fn extract_published_date(item: &Value) -> &str {
    item.get("published_at")
        .or_else(|| item.get("date"))
        .or_else(|| item.get("created_at"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

async fn generate_section_cards(
    items: &[&Value],
    source_slug: &str,
    source_id: &str,
    link_gen: &LinkGenerationService,
    section_prefix: &str,
) -> Vec<String> {
    let mut cards = Vec::new();

    for (index, rel_item) in items.iter().enumerate() {
        let title = rel_item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled");
        let excerpt = rel_item
            .get("description")
            .or_else(|| rel_item.get("excerpt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let slug = rel_item.get("slug").and_then(|v| v.as_str());
        let image = rel_item.get("image").and_then(|v| v.as_str());

        let rel_date = extract_published_date(rel_item);
        let short_date = format_short_date(rel_date);

        if let Some(slug) = slug {
            let target_url = format!("/blog/{slug}");
            let source_page = format!("/blog/{source_slug}");
            let link_position = format!("{}-{}", section_prefix, index + 1);

            let card_data = CardData {
                title,
                slug,
                description: excerpt,
                image,
                date: &short_date,
                url_prefix: "/blog",
            };

            let card_url = match link_gen
                .generate_internal_content_link(
                    &target_url,
                    source_id,
                    &source_page,
                    Some(title.to_string()),
                    Some(link_position),
                )
                .await
            {
                Ok(tracked_link) => format!("/r/{}", tracked_link.short_code),
                Err(_) => format!("/blog/{slug}"),
            };

            cards.push(generate_related_card(&card_data, &card_url));
        }
    }

    cards
}

pub async fn generate_latest_and_popular_html(
    item: &Value,
    latest: &[&Value],
    popular: &[&Value],
    db_pool: DbPool,
) -> String {
    if latest.is_empty() && popular.is_empty() {
        return String::new();
    }

    let source_slug = item
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let source_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    let link_gen = LinkGenerationService::new(db_pool);

    let latest_cards = generate_section_cards(latest, source_slug, source_id, &link_gen, "latest").await;
    let popular_cards = generate_section_cards(popular, source_slug, source_id, &link_gen, "popular").await;

    let mut sections = Vec::new();

    if !latest_cards.is_empty() {
        sections.push(format!(
            r#"<section class="related-section">
  <h3>Latest Blogs</h3>
  <div class="related-grid">{}</div>
</section>"#,
            latest_cards.join("\n")
        ));
    }

    if !popular_cards.is_empty() {
        sections.push(format!(
            r#"<section class="related-section">
  <h3>Most Popular</h3>
  <div class="related-grid">{}</div>
</section>"#,
            popular_cards.join("\n")
        ));
    }

    if sections.is_empty() {
        return String::new();
    }

    format!(
        r#"<div class="related-articles">{}</div>"#,
        sections.join("\n")
    )
}

pub async fn generate_cta_links(item: &Value, db_pool: DbPool) -> (String, String) {
    let source_slug = item
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let source_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    let link_gen = LinkGenerationService::new(db_pool);
    let source_page = format!("/blog/{source_slug}");

    let header_cta_url = match link_gen
        .generate_internal_content_link(
            "/",
            source_id,
            &source_page,
            Some("Header Chat CTA".to_string()),
            Some("header-cta".to_string()),
        )
        .await
    {
        Ok(tracked_link) => format!("/r/{}", tracked_link.short_code),
        Err(_) => "/".to_string(),
    };

    let banner_cta_url = match link_gen
        .generate_internal_content_link(
            "/",
            source_id,
            &source_page,
            Some("Banner Chat CTA".to_string()),
            Some("banner-cta".to_string()),
        )
        .await
    {
        Ok(tracked_link) => format!("/r/{}", tracked_link.short_code),
        Err(_) => "/".to_string(),
    };

    (header_cta_url, banner_cta_url)
}

pub fn generate_references_html(item: &Value) -> String {
    let links_array = match item.get("links").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return String::new(),
    };

    let cards: Vec<String> = links_array
        .iter()
        .enumerate()
        .filter_map(|(index, link)| {
            let title = link.get("title")?.as_str()?;
            let url = link.get("url")?.as_str()?;
            let domain = url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();

            Some(format!(
                r#"<article class="reference-card">
  <span class="reference-card__number">[{}]</span>
  <a href="{}" class="reference-card__title" target="_blank" rel="noopener noreferrer">{}</a>
  <span class="reference-card__domain">{}</span>
</article>"#,
                index + 1,
                url,
                title,
                domain
            ))
        })
        .collect();

    if cards.is_empty() {
        return String::new();
    }

    format!(
        r#"<section class="references">
  <h2>References &amp; Sources</h2>
  <div class="references-grid">{}</div>
</section>"#,
        cards.join("\n")
    )
}

pub async fn generate_social_content_html(_item: &Value, _db_pool: DbPool) -> String {
    String::new()
}
