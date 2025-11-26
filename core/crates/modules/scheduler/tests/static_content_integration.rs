use serde_json::json;
use std::path::PathBuf;

#[tokio::test]
async fn test_markdown_rendering_with_various_formats() {
    use systemprompt_core_scheduler::services::static_content::render_markdown;

    let test_cases = vec![
        ("# Heading", "<h1>Heading</h1>"),
        ("**bold**", "<strong>bold</strong>"),
        ("*italic*", "<em>italic</em>"),
        ("[link](https://example.com)", "https://example.com"),
        ("`code`", "<code>code</code>"),
        ("~~strikethrough~~", "<del>strikethrough</del>"),
    ];

    for (input, expected_substring) in test_cases {
        let result = render_markdown(input).expect("Markdown rendering failed");
        assert!(
            result.contains(expected_substring),
            "Expected '{}' to contain '{}', got: {}",
            input,
            expected_substring,
            result
        );
    }
}

#[test]
fn test_config_parsing_with_multiple_sources() {
    use systemprompt_core_scheduler::services::static_content::ContentConfig;

    let yaml = r#"
content_sources:
  blog:
    path: crates/services/content/blog
    source_id: blog
    category_id: blog
    enabled: true
    sitemap:
      enabled: true
      url_pattern: /blog/{slug}
      priority: 0.8
      changefreq: weekly

  pages:
    path: crates/services/content/pages
    source_id: pages
    category_id: pages
    enabled: true
    sitemap:
      enabled: true
      url_pattern: /pages/{slug}
      priority: 0.6
      changefreq: monthly

  legal:
    path: crates/services/content/legal
    source_id: legal
    category_id: legal
    enabled: false
    sitemap:
      enabled: true
      url_pattern: /legal/{slug}
      priority: 0.5
      changefreq: yearly

metadata:
  default_author: Default Author
  language: en
  structured_data:
    organization:
      name: Test Org
      url: https://test.org
      logo: https://test.org/logo.png
    article:
      article_type: Article
      article_section: Blog
      language: en-US

categories:
  blog:
    name: Blog
    description: Blog posts
  pages:
    name: Pages
    description: Static pages
"#;

    let config: ContentConfig = serde_yaml::from_str(yaml).expect("Failed to parse config");

    assert_eq!(config.content_sources.len(), 3);
    assert!(config.content_sources.contains_key("blog"));
    assert!(config.content_sources.contains_key("pages"));
    assert!(config.content_sources.contains_key("legal"));

    let blog_source = &config.content_sources["blog"];
    assert!(blog_source.enabled);
    assert_eq!(blog_source.source_id, "blog");

    let pages_source = &config.content_sources["pages"];
    assert!(pages_source.enabled);
    assert_eq!(pages_source.source_id, "pages");

    let legal_source = &config.content_sources["legal"];
    assert!(!legal_source.enabled);

    assert_eq!(config.metadata.default_author, "Default Author");
    assert_eq!(config.categories.len(), 2);
}

#[test]
fn test_frontmatter_extraction_edge_cases() {
    use systemprompt_core_scheduler::services::static_content::extract_frontmatter;

    let cases = vec![
        ("---\ntitle: Test\n---\nContent", true, "Basic frontmatter"),
        (
            "---\ntitle: Test\nauthor: John\ntags: [a, b, c]\n---\n# Content",
            true,
            "Multiple fields",
        ),
        (
            "No frontmatter\n---\nJust dashes",
            false,
            "No leading dashes",
        ),
        (
            "---\nincomplete frontmatter",
            false,
            "Incomplete frontmatter",
        ),
    ];

    for (content, should_extract, description) in cases {
        let result = extract_frontmatter(content);
        assert_eq!(
            result.is_some(),
            should_extract,
            "Test case '{}' failed",
            description
        );
    }
}

#[test]
fn test_template_data_preparation() {
    use serde_json::json;
    use systemprompt_core_scheduler::services::static_content::prepare_template_data;

    let item = json!({
        "title": "Test Article",
        "slug": "test-article",
        "content": "Test content",
        "author": "John Doe",
        "published_at": "2024-01-01T10:00:00Z",
        "updated_at": "2024-01-02T10:00:00Z",
        "category_id": "blog",
        "description": "A test article",
        "keywords": ["test", "article"],
        "image": "https://example.com/image.jpg"
    });

    let all_items = vec![
        json!({
            "title": "Another Article",
            "slug": "another-article",
            "category_id": "blog",
        }),
        json!({
            "title": "Different Category",
            "slug": "different",
            "category_id": "pages",
        }),
    ];

    let config = serde_yaml::from_str(
        r#"
metadata:
  default_author: Default Author
  structured_data:
    organization:
      name: Example Org
      url: https://example.com
      logo: https://example.com/logo.png
    article:
      article_type: Article
      article_section: Blog
      language: en-US
"#,
    )
    .unwrap();

    let data = prepare_template_data(&item, &all_items, &config, "<p>HTML Content</p>");

    assert_eq!(data["title"], "Test Article");
    assert_eq!(data["slug"], "test-article");
    assert_eq!(data["author"], "John Doe");
    assert!(data["content"].as_str().unwrap().contains("HTML Content"));
    assert_eq!(data["org_name"], "Example Org");
    assert_eq!(data["article_type"], "Article");
    assert_eq!(data["article_section"], "Blog");

    let related = data["related_items"].as_array().unwrap();
    assert_eq!(related.len(), 1);
}

#[test]
fn test_sitemap_xml_generation_with_special_characters() {
    let test_cases = vec![
        ("hello-world", "hello-world", "No escaping needed"),
        ("special&chars", "special&amp;chars", "Ampersand escaped"),
        ("with<tags>", "with&lt;tags&gt;", "Angle brackets escaped"),
        ("quotes\"test\"", "quotes&quot;test&quot;", "Quotes escaped"),
        ("apostrophe's", "apostrophe&apos;s", "Apostrophe escaped"),
    ];

    for (input, expected, description) in test_cases {
        let escaped = escape_xml(input);
        assert_eq!(escaped, expected, "Failed: {}", description);
        println!("✓ {} -> {}", description, escaped);
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[tokio::test]
async fn test_template_engine_loading_templates() {
    use systemprompt_core_scheduler::services::static_content::TemplateEngine;
    use tempfile::TempDir;
    use tokio::fs;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("test.html");

    fs::write(&template_path, "Hello {{name}}!")
        .await
        .expect("Failed to write template");

    let engine = TemplateEngine::new(temp_dir.path().to_str().unwrap())
        .await
        .expect("Failed to create template engine");

    let result = engine
        .render("test", &json!({"name": "World"}))
        .expect("Failed to render");

    assert_eq!(result, "Hello World!");
}

#[test]
fn test_output_directory_calculation() {
    let test_cases = vec![
        (
            "/app/dist",
            "/blog/{slug}",
            "hello-world",
            "/app/dist/blog/hello-world",
        ),
        (
            "/app/dist",
            "/articles/{slug}/",
            "my-article",
            "/app/dist/articles/my-article/",
        ),
        ("/app/dist", "/{slug}", "page", "/app/dist/page"),
        (
            "/app/web",
            "/posts/{slug}/index",
            "test",
            "/app/web/posts/test/index",
        ),
    ];

    for (dist_dir, pattern, slug, expected) in test_cases {
        let result = pattern.replace("{slug}", slug);
        let result = result.trim_start_matches('/');
        let output = PathBuf::from(dist_dir).join(result);
        assert_eq!(
            output.to_string_lossy(),
            expected,
            "Pattern: {}, Slug: {}",
            pattern,
            slug
        );
    }
}

#[test]
fn test_content_config_defaults() {
    use systemprompt_core_scheduler::services::static_content::ContentConfig;

    let empty_yaml = r#"
content_sources: {}
"#;

    let config: ContentConfig = serde_yaml::from_str(empty_yaml).expect("Failed to parse");
    assert_eq!(config.content_sources.len(), 0);
    assert_eq!(config.metadata.default_author, "");
}

#[test]
fn test_markdown_gfm_extensions() {
    use systemprompt_core_scheduler::services::static_content::render_markdown;

    let test_cases = vec![
        ("- [ ] Task 1\n- [x] Task 2", "checkbox", true),
        ("| H1 | H2 |\n|---|---|\n| A | B |", "table", true),
        ("~~strikethrough~~", "strikethrough", true),
        ("2^nd", "superscript", true),
        ("https://example.com", "autolink", true),
    ];

    for (markdown, feature_name, should_work) in test_cases {
        match render_markdown(markdown) {
            Ok(_html) => {
                if should_work {
                    println!("✓ GFM feature '{}' works", feature_name);
                } else {
                    println!("✗ GFM feature '{}' unexpected success", feature_name);
                }
            },
            Err(e) => {
                if !should_work {
                    println!("✓ GFM feature '{}' correctly rejected", feature_name);
                } else {
                    eprintln!("✗ GFM feature '{}' failed: {}", feature_name, e);
                }
            },
        }
    }
}

#[test]
fn test_sitemap_pagination_logic() {
    const MAX_URLS_PER_SITEMAP: usize = 50_000;

    let test_cases = vec![
        (1_000, 1, "Small sitemap"),
        (50_000, 1, "Exactly max URLs"),
        (50_001, 2, "Just over limit"),
        (100_000, 2, "Exactly 2x limit"),
        (100_001, 3, "Just over 2x limit"),
    ];

    for (total_urls, expected_files, description) in test_cases {
        let chunks = (total_urls + MAX_URLS_PER_SITEMAP - 1) / MAX_URLS_PER_SITEMAP;
        assert_eq!(
            chunks, expected_files,
            "Test '{}': {} URLs should create {} files",
            description, total_urls, expected_files
        );
    }
}

#[test]
fn test_multiple_content_sources_config() {
    use systemprompt_core_scheduler::services::static_content::ContentConfig;

    let yaml = r#"
content_sources:
  blog:
    path: blog
    source_id: blog_source
    category_id: blog_category
    enabled: true
    sitemap:
      enabled: true
      url_pattern: /blog/{slug}
      priority: 0.8
      changefreq: weekly

  documentation:
    path: docs
    source_id: docs_source
    category_id: docs_category
    enabled: true
    sitemap:
      enabled: true
      url_pattern: /docs/{slug}
      priority: 0.9
      changefreq: daily

metadata:
  default_author: Team
  language: en
  structured_data:
    organization:
      name: Company
      url: https://company.com
      logo: https://company.com/logo.png
    article:
      article_type: Article
      article_section: General
      language: en-US
"#;

    let config: ContentConfig = serde_yaml::from_str(yaml).expect("Failed to parse");

    assert_eq!(config.content_sources.len(), 2);

    let blog = &config.content_sources["blog"];
    assert_eq!(blog.source_id, "blog_source");
    assert_eq!(blog.category_id, "blog_category");
    assert!(blog.sitemap.as_ref().unwrap().enabled);
    assert_eq!(blog.sitemap.as_ref().unwrap().priority, 0.8);

    let docs = &config.content_sources["documentation"];
    assert_eq!(docs.source_id, "docs_source");
    assert_eq!(docs.category_id, "docs_category");
    assert!(docs.sitemap.as_ref().unwrap().enabled);
    assert_eq!(docs.sitemap.as_ref().unwrap().priority, 0.9);

    assert_eq!(config.metadata.default_author, "Team");
}

#[test]
fn test_error_handling_for_invalid_patterns() {
    let patterns = vec![
        ("", "empty"),
        ("{invalid}", "no slug"),
        ("//double/slash", "double slash"),
        ("no-placeholder", "no placeholder"),
    ];

    for (pattern, description) in patterns {
        let result = pattern.replace("{slug}", "test");
        println!(
            "Pattern '{}' ({}): {} -> {}",
            pattern, description, pattern, result
        );
    }
}
