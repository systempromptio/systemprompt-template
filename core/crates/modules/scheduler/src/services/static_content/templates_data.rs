use anyhow::Result;
use serde_json::{json, Value};
use systemprompt_core_database::DbPool;

use super::cards::{get_absolute_image_url, normalize_image_url};
use super::templates_html::{
    generate_cta_links, generate_latest_and_popular_html, generate_references_html,
    generate_social_content_html,
};
use super::templates_navigation::{generate_footer_html, generate_social_action_bar_html};
use super::templates_paper::{
    calculate_read_time, generate_toc_html, parse_paper_metadata, render_paper_sections_html,
};

fn format_date_pair(date_str: &str) -> (String, String) {
    if date_str.is_empty() {
        return (String::new(), String::new());
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        (
            dt.format("%B %d, %Y").to_string(),
            dt.format("%Y-%m-%d").to_string(),
        )
    } else {
        (date_str.to_string(), date_str.to_string())
    }
}

fn extract_published_date(item: &Value) -> &str {
    item.get("published_at")
        .or_else(|| item.get("date"))
        .or_else(|| item.get("created_at"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

fn extract_org_config(config: &serde_yaml::Value) -> (&str, &str, &str) {
    let org = config
        .get("metadata")
        .and_then(|m| m.get("structured_data"))
        .and_then(|s| s.get("organization"));

    let org_name = org
        .and_then(|o| o.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let org_url = org
        .and_then(|o| o.get("url"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let org_logo = org
        .and_then(|o| o.get("logo"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    (org_name, org_url, org_logo)
}

fn extract_article_config(config: &serde_yaml::Value) -> (&str, &str, &str) {
    let article = config
        .get("metadata")
        .and_then(|m| m.get("structured_data"))
        .and_then(|s| s.get("article"));

    let article_type = article
        .and_then(|a| a.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("Article");
    let article_section = article
        .and_then(|a| a.get("article_section"))
        .and_then(|v| v.as_str())
        .unwrap_or("General");
    let article_language = article
        .and_then(|a| a.get("language"))
        .and_then(|v| v.as_str())
        .unwrap_or("en-US");

    (article_type, article_section, article_language)
}

fn find_latest_items<'a>(item: &Value, all_items: &'a [Value], limit: usize) -> Vec<&'a Value> {
    let item_slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

    all_items
        .iter()
        .filter(|other| {
            let other_slug = other.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            other_slug != item_slug
        })
        .take(limit)
        .collect()
}

fn find_popular_items<'a>(
    item: &Value,
    all_items: &'a [Value],
    popular_ids: &[String],
    exclude_slugs: &[&str],
    limit: usize,
) -> Vec<&'a Value> {
    let item_slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

    let mut popular: Vec<&'a Value> = popular_ids
        .iter()
        .filter_map(|id| {
            all_items.iter().find(|item| {
                let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                item_id == id && slug != item_slug && !exclude_slugs.contains(&slug)
            })
        })
        .take(limit)
        .collect();

    if popular.len() < limit {
        let remaining = limit - popular.len();
        let popular_slugs: Vec<&str> = popular
            .iter()
            .filter_map(|p| p.get("slug").and_then(|v| v.as_str()))
            .collect();

        let fallback: Vec<&'a Value> = all_items
            .iter()
            .filter(|other| {
                let other_slug = other.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                other_slug != item_slug
                    && !exclude_slugs.contains(&other_slug)
                    && !popular_slugs.contains(&other_slug)
            })
            .take(remaining)
            .collect();

        popular.extend(fallback);
    }

    popular
}

fn prepare_paper_data(
    item: &Value,
    content_html: &str,
    absolute_image_url: &str,
    org_url: &str,
) -> (String, String, String, String) {
    let content_type = item
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if content_type != "paper" {
        return (
            absolute_image_url.to_string(),
            String::new(),
            String::new(),
            content_html.to_string(),
        );
    }

    let markdown_content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let paper_meta = parse_paper_metadata(markdown_content).unwrap_or_default();

    let hero_img = paper_meta
        .hero_image
        .as_ref()
        .map(|i| {
            if i.starts_with("http") || i.starts_with("/") {
                i.clone()
            } else {
                format!("/{i}")
            }
        })
        .unwrap_or_else(|| absolute_image_url.to_string());

    let hero_alt_text = paper_meta.hero_alt.clone().unwrap_or_else(|| {
        item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    });

    let toc = if paper_meta.toc {
        generate_toc_html(&paper_meta)
    } else {
        String::new()
    };

    let sections = render_paper_sections_html(markdown_content, &paper_meta, org_url)
        .unwrap_or_else(|_| content_html.to_string());

    (hero_img, hero_alt_text, toc, sections)
}

pub async fn prepare_template_data(
    item: &Value,
    all_items: &[Value],
    popular_ids: &[String],
    config: &serde_yaml::Value,
    web_config: &serde_yaml::Value,
    content_html: &str,
    db_pool: DbPool,
) -> Result<Value> {
    let footer_html = generate_footer_html(web_config);
    let social_action_bar_html = generate_social_action_bar_html(web_config, false);
    let social_action_bar_hero_html = generate_social_action_bar_html(web_config, true);

    let (org_name, org_url, org_logo) = extract_org_config(config);
    let (article_type, article_section, article_language) = extract_article_config(config);

    let published_date = extract_published_date(item);
    let (formatted_date, date_iso) = format_date_pair(published_date);

    let (formatted_modified, date_modified_iso) = item
        .get("updated_at")
        .and_then(|v| v.as_str())
        .map(|date_str| format_date_pair(date_str))
        .unwrap_or_else(|| (formatted_date.clone(), date_iso.clone()));

    let latest = find_latest_items(item, all_items, 6);
    let latest_slugs: Vec<&str> = latest
        .iter()
        .filter_map(|i| i.get("slug").and_then(|v| v.as_str()))
        .collect();
    let popular = find_popular_items(item, all_items, popular_ids, &latest_slugs, 6);
    let related_html =
        generate_latest_and_popular_html(item, &latest, &popular, db_pool.clone()).await;

    let (header_cta_url, banner_cta_url) = generate_cta_links(item, db_pool.clone()).await;

    let read_time = calculate_read_time(content_html);
    let references_html = generate_references_html(item);
    let social_html = generate_social_content_html(item, db_pool).await;

    let raw_image = item
        .get("image")
        .or_else(|| item.get("cover_image"))
        .and_then(|v| v.as_str());
    let featured_image = normalize_image_url(raw_image).unwrap_or_default();
    let absolute_image_url = get_absolute_image_url(raw_image, org_url).unwrap_or_default();

    let default_author = config["metadata"]["default_author"]
        .as_str()
        .unwrap_or("tyingshoelaces");
    let author = item
        .get("author")
        .and_then(|v| v.as_str())
        .filter(|a| !a.is_empty() && !a.contains("local"))
        .unwrap_or(default_author);

    let twitter_handle = web_config
        .get("branding")
        .and_then(|b| b.get("twitter_handle"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (hero_image, hero_alt, toc_html, sections_html) =
        prepare_paper_data(item, content_html, &absolute_image_url, org_url);

    Ok(json!({
        "TITLE": item.get("title").unwrap_or(&json!("")),
        "DESCRIPTION": item.get("description")
            .or_else(|| item.get("excerpt"))
            .unwrap_or(&json!("")),
        "AUTHOR": author,
        "DATE": formatted_date,
        "DATE_PUBLISHED": formatted_date,
        "DATE_MODIFIED": formatted_modified,
        "DATE_ISO": date_iso,
        "DATE_MODIFIED_ISO": date_modified_iso,
        "READ_TIME": read_time,
        "KEYWORDS": item.get("keywords")
            .or_else(|| item.get("tags"))
            .unwrap_or(&json!("")),
        "IMAGE": absolute_image_url,
        "FEATURED_IMAGE": featured_image,
        "CONTENT": content_html,
        "SLUG": item.get("slug").unwrap_or(&json!("")),
        "ORG_NAME": org_name,
        "ORG_URL": org_url,
        "ORG_LOGO": org_logo,
        "TWITTER_HANDLE": twitter_handle,
        "ARTICLE_TYPE": article_type,
        "ARTICLE_SECTION": article_section,
        "ARTICLE_LANGUAGE": article_language,
        "HEADER_CTA_URL": header_cta_url,
        "BANNER_CTA_URL": banner_cta_url,
        "SOCIAL_ACTION_BAR": social_action_bar_html,
        "SOCIAL_ACTION_BAR_HERO": social_action_bar_hero_html,
        "RELATED_CONTENT": related_html,
        "REFERENCES": references_html,
        "SOCIAL_CONTENT": social_html,
        "FOOTER_NAV": footer_html,
        "DISPLAY_SITENAME": web_config.get("branding")
            .and_then(|b| b.get("display_sitename"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        "HERO_IMAGE": hero_image,
        "HERO_ALT": hero_alt,
        "TOC_HTML": toc_html,
        "SECTIONS_HTML": sections_html,
    }))
}
