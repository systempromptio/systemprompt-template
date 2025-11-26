import { writeFileSync, mkdirSync, readFileSync } from 'fs';
import { join, dirname } from 'path';
import { marked } from 'marked';
import { fileURLToPath } from 'url';
import yaml from 'js-yaml';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load content configuration
function loadContentConfig() {
  const configPath = join(__dirname, '../../../crates/services/content/config.yml');
  const configContent = readFileSync(configPath, 'utf8');
  return yaml.load(configContent);
}

// 1. Fetch blog posts from database API
async function fetchBlogPosts() {
  const apiUrl = process.env.VITE_API_URL || 'http://localhost:8080';
  const response = await fetch(`${apiUrl}/api/v1/content/blog/json`);

  if (!response.ok) {
    throw new Error('Failed to fetch blog posts. Is API running?');
  }

  const data = await response.json();
  return data;
}

// 2. Load HTML template
function loadTemplate(templateName) {
  const templatePath = join(__dirname, '../templates', templateName);
  return readFileSync(templatePath, 'utf8');
}

// 3. Render single blog post to HTML
function renderBlogPost(post, template, config) {
  // Render markdown to HTML
  const contentHtml = marked.parse(post.content || '');

  // Format dates
  const publishedDate = post.published_at || post.date || post.created_at || new Date().toISOString();
  const modifiedDate = post.updated_at || publishedDate;

  // Get structured data config
  const sd = config.metadata?.structured_data || {};
  const orgConfig = sd.organization || {};
  const articleConfig = sd.article || {};

  // Inject into template
  return template
    .replace(/\{\{TITLE\}\}/g, escapeHtml(post.title || ''))
    .replace(/\{\{DESCRIPTION\}\}/g, escapeHtml(post.description || post.excerpt || ''))
    .replace(/\{\{AUTHOR\}\}/g, escapeHtml(post.author || config.metadata?.default_author || 'Anonymous'))
    .replace(/\{\{DATE\}\}/g, escapeHtml(publishedDate))
    .replace(/\{\{DATE_PUBLISHED\}\}/g, escapeHtml(publishedDate))
    .replace(/\{\{DATE_MODIFIED\}\}/g, escapeHtml(modifiedDate))
    .replace(/\{\{KEYWORDS\}\}/g, escapeHtml(post.keywords || post.tags || ''))
    .replace(/\{\{IMAGE\}\}/g, escapeHtml(post.image || post.cover_image || ''))
    .replace(/\{\{CONTENT\}\}/g, contentHtml)
    .replace(/\{\{SLUG\}\}/g, escapeHtml(post.slug || ''))
    .replace(/\{\{ORG_NAME\}\}/g, escapeHtml(orgConfig.name || ''))
    .replace(/\{\{ORG_URL\}\}/g, escapeHtml(orgConfig.url || ''))
    .replace(/\{\{ORG_LOGO\}\}/g, escapeHtml(orgConfig.logo || ''))
    .replace(/\{\{ARTICLE_TYPE\}\}/g, articleConfig.type || 'BlogPosting')
    .replace(/\{\{ARTICLE_SECTION\}\}/g, escapeHtml(articleConfig.article_section || 'Technology'))
    .replace(/\{\{ARTICLE_LANGUAGE\}\}/g, articleConfig.language || 'en-US');
}

// 4. Write HTML file
function writeBlogPost(slug, html) {
  const outputDir = join(__dirname, '../dist/blog', slug);
  mkdirSync(outputDir, { recursive: true });

  const outputPath = join(outputDir, 'index.html');
  writeFileSync(outputPath, html);

  console.log(`  ✅ Generated: /blog/${slug}/index.html`);
}

// 5. Generate blog list page
function generateBlogList(posts, config) {
  const sd = config.metadata?.structured_data || {};
  const orgConfig = sd.organization || {};
  const blogConfig = sd.blog || {};

  const postsHtml = posts.map(post => {
    const formattedDate = post.date || post.created_at || '';
    const excerpt = post.excerpt || post.description || '';
    const author = post.author || config.metadata?.default_author || 'Anonymous';

    return `
    <article class="blog-card">
      <h2><a href="/blog/${post.slug}">${escapeHtml(post.title || '')}</a></h2>
      <p>${escapeHtml(excerpt)}</p>
      <div class="meta">
        <span>${escapeHtml(author)}</span>
        <span>${escapeHtml(formattedDate)}</span>
      </div>
    </article>
  `}).join('');

  // Generate structured data for blog posts list
  const blogPostsJson = JSON.stringify(posts.map(post => ({
    "@type": "BlogPosting",
    "headline": post.title,
    "url": `${orgConfig.url || ''}/blog/${post.slug}`,
    "datePublished": post.published_at || post.date || post.created_at,
    "author": {
      "@type": "Person",
      "name": post.author || config.metadata?.default_author || 'Anonymous'
    }
  })), null, 2);

  const listTemplate = loadTemplate('blog-list.html');
  return listTemplate
    .replace('{{POSTS}}', postsHtml)
    .replace('{{BLOG_POSTS_JSON}}', blogPostsJson)
    .replace(/\{\{ORG_NAME\}\}/g, escapeHtml(orgConfig.name || ''))
    .replace(/\{\{ORG_URL\}\}/g, escapeHtml(orgConfig.url || ''))
    .replace(/\{\{ORG_LOGO\}\}/g, escapeHtml(orgConfig.logo || ''))
    .replace(/\{\{BLOG_TYPE\}\}/g, blogConfig.type || 'Blog')
    .replace(/\{\{BLOG_NAME\}\}/g, escapeHtml(blogConfig.name || 'Blog'))
    .replace(/\{\{BLOG_URL\}\}/g, escapeHtml(blogConfig.url || ''))
    .replace(/\{\{BLOG_DESCRIPTION\}\}/g, escapeHtml(blogConfig.description || ''))
    .replace(/\{\{BLOG_LANGUAGE\}\}/g, blogConfig.language || 'en-US');
}

// 6. Main execution
async function main() {
  console.log('📄 Pre-rendering blog posts as static HTML...');

  // Load configuration
  const config = loadContentConfig();
  console.log(`   Loaded config from: crates/services/content/config.yml`);

  const posts = await fetchBlogPosts();
  console.log(`   Found ${posts.length} blog posts`);

  const template = loadTemplate('blog-post.html');

  // Render each blog post
  for (const post of posts) {
    const html = renderBlogPost(post, template, config);
    writeBlogPost(post.slug, html);
  }

  // Render blog list page
  const listHtml = generateBlogList(posts, config);
  const listDir = join(__dirname, '../dist/blog');
  mkdirSync(listDir, { recursive: true });
  writeFileSync(join(listDir, 'index.html'), listHtml);
  console.log(`  ✅ Generated: /blog/index.html`);

  console.log(`✅ Pre-rendered ${posts.length} blog posts as static HTML with structured data`);
}

function escapeHtml(text) {
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

main().catch(error => {
  console.error('❌ Pre-rendering failed:', error);
  process.exit(1);
});
