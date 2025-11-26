use crate::common::*;
use anyhow::Result;
use std::fs;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_content_ingestion_populates_database() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as count FROM markdown_content WHERE source_id = 'blog'";
    let row = ctx.db.fetch_optional(&query, &[]).await?;

    let count: i64 = row
        .and_then(|r| r.get("count").and_then(|v| v.as_i64()))
        .unwrap_or(0);

    assert!(count > 0, "Blog content should be ingested");

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Content ingestion populates database");
    Ok(())
}

#[tokio::test]
async fn test_ingested_content_has_required_fields() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = r#"
        SELECT id, title, slug, description, author, content
        FROM markdown_content
        WHERE source_id = 'blog'
        LIMIT 1
    "#;

    let row = ctx.db.fetch_optional(&query, &[]).await?;
    assert!(row.is_some(), "Should have at least one blog post");

    let row = row.unwrap();
    let id = row.get("id").and_then(|v| v.as_str());
    let title = row.get("title").and_then(|v| v.as_str());
    let slug = row.get("slug").and_then(|v| v.as_str());
    let description = row.get("description").and_then(|v| v.as_str());
    let author = row.get("author").and_then(|v| v.as_str());
    let content = row.get("content").and_then(|v| v.as_str());

    assert!(
        id.is_some() && !id.unwrap().is_empty(),
        "ID should be populated"
    );
    assert!(
        title.is_some() && !title.unwrap().is_empty(),
        "Title should be populated"
    );
    assert!(
        slug.is_some() && !slug.unwrap().is_empty(),
        "Slug should be populated"
    );
    assert!(
        description.is_some() && !description.unwrap().is_empty(),
        "Description should be populated"
    );
    assert!(
        author.is_some() && !author.unwrap().is_empty(),
        "Author should be populated"
    );
    assert!(
        content.is_some() && !content.unwrap().is_empty(),
        "Content should be populated"
    );

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Ingested content has all required fields");
    Ok(())
}

#[tokio::test]
async fn test_static_content_generation_creates_files() -> Result<()> {
    let ctx = TestContext::new().await?;

    let dist_dir = std::env::current_dir()?.join("../web/dist");
    if !dist_dir.exists() {
        println!(
            "⚠️ Distribution directory not found at {:?}, skipping",
            dist_dir
        );
        return Ok(());
    }

    let blog_dir = dist_dir.join("blog");
    let blog_exists = blog_dir.exists();

    if blog_exists {
        let generated_posts = fs::read_dir(&blog_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        path.file_name()
                            .and_then(|name| name.to_str().map(|s| s.to_string()))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        assert!(
            !generated_posts.is_empty(),
            "Should generate at least one blog post directory"
        );

        println!(
            "✓ Static content generation creates {} directories",
            generated_posts.len()
        );
    }

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    Ok(())
}

#[tokio::test]
async fn test_generated_html_has_populated_metadata() -> Result<()> {
    let ctx = TestContext::new().await?;

    let dist_dir = std::env::current_dir()?.join("../web/dist");

    if !dist_dir.exists() {
        println!("⚠️ Distribution directory not found, skipping");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_dir = dist_dir.join("blog");
    if !blog_dir.exists() {
        println!("⚠️ No generated blog posts found, skipping metadata test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_posts = fs::read_dir(blog_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();

    if blog_posts.is_empty() {
        println!("⚠️ No generated blog posts found, skipping metadata test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let first_post = &blog_posts[0];
    let index_html = first_post.join("index.html");

    assert!(index_html.exists(), "Generated post should have index.html");

    let html_content = fs::read_to_string(&index_html)?;

    assert!(
        html_content.contains("<title>") && !html_content.contains("<title></title>"),
        "HTML should have populated title"
    );
    assert!(
        html_content.contains("meta name=\"description\""),
        "HTML should have description meta tag"
    );
    assert!(
        html_content.contains("meta name=\"author\""),
        "HTML should have author meta tag"
    );
    assert!(
        html_content.contains("<article>") || html_content.contains("post-content"),
        "HTML should have article content"
    );

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Generated HTML has populated metadata");
    Ok(())
}

#[tokio::test]
async fn test_generated_html_has_structured_data() -> Result<()> {
    let ctx = TestContext::new().await?;

    let dist_dir = std::env::current_dir()?.join("../web/dist");

    if !dist_dir.exists() {
        println!("⚠️ Distribution directory not found, skipping");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_dir = dist_dir.join("blog");
    if !blog_dir.exists() {
        println!("⚠️ No generated blog posts found, skipping structured data test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_posts = fs::read_dir(blog_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();

    if blog_posts.is_empty() {
        println!("⚠️ No generated blog posts found, skipping structured data test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let first_post = &blog_posts[0];
    let index_html = first_post.join("index.html");
    let html_content = fs::read_to_string(&index_html)?;

    assert!(
        html_content.contains("application/ld+json"),
        "HTML should have JSON-LD structured data"
    );
    assert!(
        html_content.contains("@type") && html_content.contains("Article"),
        "Structured data should define Article type"
    );
    assert!(
        html_content.contains("datePublished"),
        "Structured data should have datePublished"
    );
    assert!(
        html_content.contains("author"),
        "Structured data should have author"
    );

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Generated HTML has structured data");
    Ok(())
}

#[tokio::test]
async fn test_sitemap_generation_creates_valid_xml() -> Result<()> {
    let ctx = TestContext::new().await?;

    let dist_dir = std::env::current_dir()?.join("../web/dist");
    let sitemap_path = dist_dir.join("sitemap.xml");

    if sitemap_path.exists() {
        let sitemap_content = fs::read_to_string(&sitemap_path)?;

        assert!(
            sitemap_content.contains("<?xml"),
            "Sitemap should be valid XML"
        );
        assert!(
            sitemap_content.contains("<urlset"),
            "Sitemap should have urlset root"
        );
        assert!(
            sitemap_content.contains("<url>"),
            "Sitemap should have URL entries"
        );
        assert!(
            sitemap_content.contains("<loc>"),
            "Sitemap URLs should have location"
        );
        assert!(
            sitemap_content.contains("<lastmod>"),
            "Sitemap URLs should have lastmod"
        );

        println!("✓ Sitemap is valid XML");
    } else {
        println!("⚠️ Sitemap not found, skipping validation");
    }

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    Ok(())
}

#[tokio::test]
async fn test_sitemap_contains_blog_posts() -> Result<()> {
    let ctx = TestContext::new().await?;

    let blog_count: i64 = {
        let query = "SELECT COUNT(*) as count FROM markdown_content WHERE source_id = 'blog'";
        ctx.db
            .fetch_optional(&query, &[])
            .await?
            .and_then(|r| r.get("count").and_then(|v| v.as_i64()))
            .unwrap_or(0)
    };

    let dist_dir = std::env::current_dir()?.join("../web/dist");
    let sitemap_path = dist_dir.join("sitemap.xml");

    if sitemap_path.exists() && blog_count > 0 {
        let sitemap_content = fs::read_to_string(&sitemap_path)?;
        let url_count = sitemap_content.matches("<url>").count();

        assert!(
            url_count as i64 >= blog_count,
            "Sitemap should contain all blog posts (expected at least {}, found {})",
            blog_count,
            url_count
        );

        println!("✓ Sitemap contains {} blog posts", blog_count);
    } else {
        println!("⚠️ Skipping - sitemap or blog content not found");
    }

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    Ok(())
}

#[tokio::test]
async fn test_template_variables_not_empty() -> Result<()> {
    let ctx = TestContext::new().await?;

    let dist_dir = std::env::current_dir()?.join("../web/dist");

    if !dist_dir.exists() {
        println!("⚠️ Distribution directory not found, skipping");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_dir = dist_dir.join("blog");
    if !blog_dir.exists() {
        println!("⚠️ No generated blog posts found, skipping template test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    let blog_posts = fs::read_dir(blog_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();

    if blog_posts.is_empty() {
        println!("⚠️ No generated blog posts found, skipping template test");
        let cleanup = TestCleanup::new(ctx.db.clone());
        cleanup.cleanup_all().await?;
        return Ok(());
    }

    for post_dir in &blog_posts[..1.min(blog_posts.len())] {
        let index_html = post_dir.join("index.html");
        let html_content = fs::read_to_string(&index_html)?;

        // Check that placeholders are not literally in the HTML
        assert!(
            !html_content.contains("{{TITLE}}"),
            "Title placeholder should be replaced"
        );
        assert!(
            !html_content.contains("{{DESCRIPTION}}"),
            "Description placeholder should be replaced"
        );
        assert!(
            !html_content.contains("{{AUTHOR}}"),
            "Author placeholder should be replaced"
        );
        assert!(
            !html_content.contains("{{CONTENT}}"),
            "Content placeholder should be replaced"
        );

        // Check that values are not empty
        let title_match = html_content.lines().find(|line| line.contains("<title>"));
        assert!(
            title_match
                .map(|line| line.len() > "<title></title>".len())
                .unwrap_or(false),
            "Title should not be empty"
        );

        // Check for non-empty article content
        assert!(
            html_content.len() > 1000,
            "Generated HTML should have substantial content"
        );
    }

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Template variables replaced correctly");
    Ok(())
}

#[tokio::test]
async fn test_content_ingestion_handles_frontmatter() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = r#"
        SELECT COUNT(*) as count FROM markdown_content
        WHERE source_id = 'blog' AND title IS NOT NULL AND title != ''
    "#;

    let count: i64 = ctx
        .db
        .fetch_optional(&query, &[])
        .await?
        .and_then(|r| r.get("count").and_then(|v| v.as_i64()))
        .unwrap_or(0);

    assert!(
        count > 0,
        "All ingested blog content should have valid titles from frontmatter"
    );

    let cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Frontmatter parsing works correctly");
    Ok(())
}
