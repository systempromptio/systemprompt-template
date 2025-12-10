fn get_social_icon(platform_type: &str) -> &'static str {
    match platform_type {
        "github" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>"#
        },
        "twitter" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>"#
        },
        "linkedin" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"/></svg>"#
        },
        "email" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>"#
        },
        "share" => {
            r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92-1.31-2.92-2.92-2.92z"/></svg>"#
        },
        _ => "",
    }
}

fn build_social_link(platform_type: &str, href: &str) -> String {
    let icon = get_social_icon(platform_type);
    if icon.is_empty() {
        return String::new();
    }

    let platform_label = platform_type
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + &platform_type[1..])
        .unwrap_or_default();

    format!(
        r#"<a href="{}" target="_blank" rel="noopener noreferrer" class="social-action-bar__link social-action-bar__link--{}" aria-label="Follow on {}">{}</a>"#,
        href, platform_type, platform_label, icon
    )
}

fn build_share_button() -> String {
    let share_icon = get_social_icon("share");
    format!(
        r#"<button type="button" class="social-action-bar__link social-action-bar__link--share" aria-label="Share this page" onclick="if(navigator.share){{navigator.share({{title:document.title,url:window.location.href}})}}else{{navigator.clipboard.writeText(window.location.href).then(()=>alert('Link copied!'))}}">{}</button>"#,
        share_icon
    )
}

pub fn generate_social_action_bar_html(web_config: &serde_yaml::Value, is_hero: bool) -> String {
    let action_bar_config = web_config.get("social_action_bar");
    let social_links = web_config
        .get("navigation")
        .and_then(|n| n.get("social"))
        .and_then(|s| s.as_sequence());

    let label = action_bar_config
        .and_then(|c| c.get("label"))
        .and_then(|l| l.as_str())
        .unwrap_or("Follow:");

    let platforms = action_bar_config
        .and_then(|c| c.get("platforms"))
        .and_then(|p| p.as_sequence());

    let enable_share = action_bar_config
        .and_then(|c| c.get("enable_share"))
        .and_then(|e| e.as_bool())
        .unwrap_or(true);

    let hero_class = if is_hero {
        " social-action-bar--hero"
    } else {
        ""
    };

    let mut links_html = Vec::new();

    if let (Some(platforms), Some(social_links)) = (platforms, social_links) {
        for platform in platforms {
            let platform_type = platform.get("type").and_then(|t| t.as_str()).unwrap_or("");

            if let Some(social_link) = social_links
                .iter()
                .find(|link| link.get("type").and_then(|t| t.as_str()) == Some(platform_type))
            {
                let href = social_link
                    .get("href")
                    .and_then(|h| h.as_str())
                    .unwrap_or("#");
                let link_html = build_social_link(platform_type, href);
                if !link_html.is_empty() {
                    links_html.push(link_html);
                }
            }
        }
    }

    if enable_share {
        links_html.push(build_share_button());
    }

    if links_html.is_empty() {
        return String::new();
    }

    format!(
        r#"<div class="social-action-bar{}">
  <span class="social-action-bar__label">{}</span>
  {}
</div>"#,
        hero_class,
        label,
        links_html.join("\n  ")
    )
}

fn build_footer_section(section_name: &str, links: &serde_yaml::Value) -> Option<String> {
    let section_title = section_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + &section_name[1..])
        .unwrap_or_else(|| "Links".to_string());

    let links_seq = links.as_sequence()?;

    let link_items: Vec<String> = links_seq
        .iter()
        .filter_map(|link| {
            let path = link.get("path")?.as_str()?;
            let label = link.get("label")?.as_str()?;
            Some(format!(r#"<li><a href="{}">{}</a></li>"#, path, label))
        })
        .collect();

    if link_items.is_empty() {
        return None;
    }

    Some(format!(
        r#"<div class="footer-nav__section">
          <h4>{}</h4>
          <ul class="footer-nav__links">
            {}
          </ul>
        </div>"#,
        section_title,
        link_items.join("\n            ")
    ))
}

fn build_footer_social_html(social_links: &[serde_yaml::Value]) -> String {
    let social_items: Vec<String> = social_links
        .iter()
        .filter_map(|link| {
            let href = link.get("href")?.as_str()?;
            let label = link.get("label")?.as_str()?;
            let link_type = link.get("type")?.as_str()?;

            let icon = match link_type {
                "github" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>"#,
                "twitter" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>"#,
                "email" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>"#,
                "linkedin" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"/></svg>"#,
                _ => "",
            };

            Some(format!(
                r#"<a href="{}" target="_blank" rel="noopener noreferrer">{}{}</a>"#,
                href, icon, label
            ))
        })
        .collect();

    if social_items.is_empty() {
        return String::new();
    }

    format!(
        r#"<div class="footer-social">
        {}
      </div>"#,
        social_items.join("\n        ")
    )
}

pub fn generate_footer_html(web_config: &serde_yaml::Value) -> String {
    let navigation = web_config.get("navigation");
    let footer = navigation.and_then(|n| n.get("footer"));
    let social = navigation.and_then(|n| n.get("social"));

    let mut sections_html = Vec::new();

    if let Some(footer_config) = footer {
        if let Some(mapping) = footer_config.as_mapping() {
            for (section_name, links) in mapping {
                let section_name_str = section_name.as_str().unwrap_or("Links");
                if let Some(section_html) = build_footer_section(section_name_str, links) {
                    sections_html.push(section_html);
                }
            }
        }
    }

    let social_html = social
        .and_then(|s| s.as_sequence())
        .map(|links| build_footer_social_html(links))
        .unwrap_or_default();

    let nav_html = if sections_html.is_empty() {
        String::new()
    } else {
        format!(
            r#"<nav class="footer-nav">
        {}
      </nav>"#,
            sections_html.join("\n        ")
        )
    };

    format!(
        r#"{}

      {}

      <div class="footer-meta">
        <p>&copy; 2025 tyingshoelaces. Built with AI agents.</p>
      </div>"#,
        nav_html, social_html
    )
}
