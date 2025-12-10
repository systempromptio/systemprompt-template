use quick_xml::de::from_str;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

use super::web_build::{BuildError, Result};

#[derive(Debug, Deserialize)]
struct Urlset {
    url: Vec<UrlEntry>,
}

#[derive(Debug, Deserialize)]
struct UrlEntry {
    loc: String,
}

#[derive(Debug)]
struct ValidationError {
    url: String,
    path: String,
    expected_file: String,
}

pub async fn validate_build(web_dir: &PathBuf) -> Result<()> {
    println!("\x1b[36m  → Validating build output...\x1b[0m");

    let dist_dir = web_dir.join("dist");

    if !dist_dir.exists() {
        return Err(BuildError::ValidationFailed(
            "dist directory not found".to_string(),
        ));
    }

    let index_html = dist_dir.join("index.html");
    if !index_html.exists() {
        return Err(BuildError::ValidationFailed(
            "index.html not found in dist".to_string(),
        ));
    }

    let sitemap_path = dist_dir.join("sitemap.xml");
    if sitemap_path.exists() {
        validate_sitemap(&dist_dir, &sitemap_path).await?;
    } else {
        println!("\x1b[33m    Warning: sitemap.xml not found (skipping sitemap validation)\x1b[0m");
    }

    println!("\x1b[32m  ✓ Build validation complete\x1b[0m");
    Ok(())
}

async fn validate_sitemap(dist_dir: &PathBuf, sitemap_path: &PathBuf) -> Result<()> {
    println!("\n\x1b[1m Validating sitemap.xml...\x1b[0m");

    let sitemap_xml = fs::read_to_string(sitemap_path)
        .await
        .map_err(|e| BuildError::ValidationFailed(format!("Failed to read sitemap: {e}")))?;

    let urlset: Urlset = from_str(&sitemap_xml)
        .map_err(|e| BuildError::ValidationFailed(format!("Failed to parse sitemap XML: {e}")))?;

    let (valid_count, missing_count, errors) = validate_urls(&urlset.url, dist_dir);
    print_validation_summary(urlset.url.len(), valid_count, missing_count, &errors)?;

    Ok(())
}

fn validate_urls(urls: &[UrlEntry], dist_dir: &PathBuf) -> (usize, usize, Vec<ValidationError>) {
    let mut valid_urls = 0;
    let mut missing_urls = 0;
    let mut errors: Vec<ValidationError> = Vec::new();

    for entry in urls {
        let url = &entry.loc;
        let path = match extract_path_from_url(url) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let html_path = if path == "/" {
            dist_dir.join("index.html")
        } else {
            dist_dir
                .join(path.trim_start_matches('/'))
                .join("index.html")
        };

        if html_path.exists() {
            valid_urls += 1;
            println!("    \x1b[32m[OK]\x1b[0m {path}");
        } else {
            missing_urls += 1;
            errors.push(ValidationError {
                url: url.clone(),
                path: path.clone(),
                expected_file: html_path.display().to_string(),
            });
            println!("    \x1b[31m[MISSING]\x1b[0m {path} (missing: {path}/index.html)");
        }
    }

    (valid_urls, missing_urls, errors)
}

fn print_validation_summary(
    total: usize,
    valid: usize,
    missing: usize,
    errors: &[ValidationError],
) -> Result<()> {
    println!("\n{}", "=".repeat(60));
    println!("\x1b[1mVALIDATION SUMMARY\x1b[0m");
    println!("{}", "=".repeat(60));
    println!("Total URLs:    {total}");
    println!("Valid URLs:    {valid} \x1b[32m[OK]\x1b[0m");
    println!("Missing URLs:  {missing} \x1b[31m[MISSING]\x1b[0m");
    println!("{}", "=".repeat(60));

    if missing > 0 {
        print_validation_errors(errors);
        return Err(BuildError::ValidationFailed(format!(
            "{missing} URLs missing corresponding HTML files"
        )));
    }

    println!("\n\x1b[1;32m[OK] All sitemap URLs are valid!\x1b[0m\n");
    Ok(())
}

fn print_validation_errors(errors: &[ValidationError]) {
    println!("\n\x1b[1;31m[WARNING] VALIDATION FAILED\x1b[0m\n");
    println!("\x1b[33mThe following URLs in sitemap.xml do not have corresponding files:\n\x1b[0m");

    for error in errors {
        println!("  URL:      {}", error.url);
        println!("  Path:     {}", error.path);
        println!("  Expected: {}", error.expected_file);
        println!();
    }

    println!("\x1b[33mThis typically means:\x1b[0m");
    println!("  1. Sitemap generation is using filenames instead of frontmatter slugs");
    println!("  2. Prerendering failed or was not run");
    println!("  3. There is a mismatch between sitemap and prerender scripts");
    println!("\n\x1b[33mRecommended fixes:\x1b[0m");
    println!("  1. Ensure sitemap generation reads frontmatter slugs");
    println!("  2. Check that prerendering job completed successfully");
    println!("  3. Check that markdown frontmatter has correct 'slug' field");
}

fn extract_path_from_url(url: &str) -> Result<String> {
    if let Some(pos) = url.find("://") {
        if let Some(slash_pos) = url[pos + 3..].find('/') {
            return Ok(url[pos + 3 + slash_pos..].to_string());
        }
    }

    if url.starts_with('/') {
        return Ok(url.to_string());
    }

    Err(BuildError::ValidationFailed(format!(
        "Invalid URL format: {url}"
    )))
}
