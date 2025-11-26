import 'dotenv/config';
import { writeFileSync, readdirSync, statSync, existsSync, readFileSync } from 'fs';
import { join, dirname, basename, extname } from 'path';
import { fileURLToPath } from 'url';
import { load as parseYaml } from 'js-yaml';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Only use localhost if explicitly set (for development)
const apiUrl = process.env.VITE_API_URL || process.env.VITE_API_BASE_URL || process.env.API_URL || '';
const today = new Date().toISOString().split('T')[0];

// Constants
const MAX_URLS_PER_SITEMAP = 50000;
const MAX_SITEMAP_SIZE_BYTES = 52428800; // 50MB

function extractSlugFromPath(filePath, basePath) {
  const relativePath = filePath.replace(basePath, '').replace(/^\//, '');
  const withoutExt = relativePath.replace(/\.md$/, '').replace(/\/index$/, '');
  return withoutExt || 'index';
}

function extractSlugFromFrontmatter(filePath) {
  try {
    const content = readFileSync(filePath, 'utf8');

    // Match frontmatter between --- delimiters
    const frontmatterMatch = content.match(/^---\s*\n([\s\S]*?)\n---/);
    if (!frontmatterMatch) {
      return null;
    }

    const frontmatter = parseYaml(frontmatterMatch[1]);
    return frontmatter?.slug || null;
  } catch (error) {
    console.warn(`   ⚠ Failed to parse frontmatter from ${filePath}: ${error.message}`);
    return null;
  }
}

function extractFrontmatter(filePath) {
  try {
    const content = readFileSync(filePath, 'utf8');

    // Match frontmatter between --- delimiters
    const frontmatterMatch = content.match(/^---\s*\n([\s\S]*?)\n---/);
    if (!frontmatterMatch) {
      return {};
    }

    return parseYaml(frontmatterMatch[1]) || {};
  } catch (error) {
    console.warn(`   ⚠ Failed to parse frontmatter from ${filePath}: ${error.message}`);
    return {};
  }
}


function shouldIncludeInSitemap(filePath, basePath) {
  // Extract filename and relative path
  const filename = basename(filePath);
  const relativePath = filePath.replace(basePath, '').replace(/^\//, '');

  // Exclude template and guideline files (special documents, not content)
  if (filename === 'template.md' || filename === 'guideline.md') {
    return false;
  }

  // Exclude README files
  if (filename === 'README.md') {
    return false;
  }

  // Exclude root-level index.md files (directory placeholders)
  // But allow subdirectory/index.md files (blog post structure)
  if (filename === 'index.md' && !relativePath.includes('/')) {
    return false;
  }

  // Extract frontmatter
  const frontmatter = extractFrontmatter(filePath);

  // If public field is explicitly set, use it
  if (frontmatter.public !== undefined) {
    return frontmatter.public === true;
  }

  // Default behavior based on content type
  const type = frontmatter.type || '';
  const status = frontmatter.status || '';

  // Exclude specific content types
  if (type === 'topic' || type === 'documentation' || type === 'social' || type === 'template') {
    return false;
  }

  // Exclude draft/active status (not public-ready)
  if (status === 'draft' || status === 'active') {
    return false;
  }

  // Default to true for content without explicit exclusion
  return true;
}

async function fetchFromDatabase(source) {
  const response = await fetch(`${apiUrl}/api/v1/content/${source.source_id}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${source.source_id}: ${response.status} ${response.statusText}`);
  }
  const data = await response.json();

  if (!Array.isArray(data)) {
    throw new Error(`API returned invalid data for ${source.source_id}: expected array, got ${typeof data}`);
  }

  return data.map(item => ({
    slug: item.slug,
    updated_at: item.updated_at || item.published_at || today,
    published_at: item.published_at || item.updated_at || today,
    title: item.title || item.slug
  }));
}

function fetchFromFilesystem(source) {
  const blogRoot = join(__dirname, '../../../');
  const fullPath = join(blogRoot, source.path);

  if (!existsSync(fullPath)) {
    throw new Error(`Filesystem path does not exist: ${fullPath}`);
  }

  const files = [];
  function scanDirectory(dir) {
    const items = readdirSync(dir);

    for (const item of items) {
      const itemPath = join(dir, item);
      const stat = statSync(itemPath);

      if (stat.isDirectory()) {
        scanDirectory(itemPath);
      } else if (item.endsWith('.md')) {
        // Check if this file should be included in sitemap
        if (!shouldIncludeInSitemap(itemPath, fullPath)) {
          continue;
        }

        // Try to read slug from frontmatter first, fall back to path
        const frontmatterSlug = extractSlugFromFrontmatter(itemPath);
        const slug = frontmatterSlug || extractSlugFromPath(itemPath, fullPath);

        // Try to read last_modified from frontmatter, fall back to file mtime, then today's date
        const frontmatter = extractFrontmatter(itemPath);
        let updated_at = frontmatter.last_modified;

        if (!updated_at) {
          updated_at = stat.mtime.toISOString().split('T')[0];
        }

        files.push({
          slug,
          updated_at,
          published_at: frontmatter.published_at || updated_at,
          title: frontmatter.title || slug
        });
      }
    }
  }

  scanDirectory(fullPath);
  return files;
}

async function fetchContentItems(source) {
  switch (source.sitemap.fetch_from) {
    case 'database':
      // Try to fetch from database, fall back to filesystem if API is unavailable
      try {
        return await fetchFromDatabase(source);
      } catch (error) {
        console.log(`   ⚠ Database fetch failed (API unavailable), falling back to filesystem for ${source.source_id}`);
        // Fall back to filesystem - use the same path pattern
        return fetchFromFilesystem(source);
      }
    case 'filesystem':
      return fetchFromFilesystem(source);
    default:
      throw new Error(`Unknown fetch_from type: ${source.sitemap.fetch_from}`);
  }
}

function generateSitemapXml(routes) {
  const urlEntries = routes.map(route => `  <url>
    <loc>${route.loc}</loc>
    <lastmod>${route.lastmod}</lastmod>
    <changefreq>${route.changefreq}</changefreq>
    <priority>${route.priority}</priority>
  </url>`).join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urlEntries}
</urlset>`;
}

function generateSitemapIndex(sitemapFiles) {
  const entries = sitemapFiles.map(file => `  <sitemap>
    <loc>${file.loc}</loc>
    <lastmod>${file.lastmod}</lastmod>
  </sitemap>`).join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${entries}
</sitemapindex>`;
}

function splitSitemaps(routes, maxUrls = MAX_URLS_PER_SITEMAP) {
  if (routes.length <= maxUrls) {
    return [{ urls: routes, index: 1, total: 1 }];
  }

  const sitemaps = [];
  for (let i = 0; i < routes.length; i += maxUrls) {
    const index = sitemaps.length + 1;
    sitemaps.push({
      urls: routes.slice(i, i + maxUrls),
      index,
      total: Math.ceil(routes.length / maxUrls)
    });
  }
  return sitemaps;
}

function getNewestDate(items) {
  if (!items || items.length === 0) return today;
  const dates = items.map(item => item.updated_at || item.published_at || today);
  return dates.sort().reverse()[0];
}

async function generateSitemap() {
  const blogRoot = join(__dirname, '../../../');

  // Read content config
  const contentConfigPath = join(blogRoot, 'crates/services/content/config.yml');
  if (!existsSync(contentConfigPath)) {
    console.error('❌ Content config not found:', contentConfigPath);
    process.exit(1);
  }

  const contentConfig = parseYaml(readFileSync(contentConfigPath, 'utf8'));

  // Read web metadata for baseUrl
  const metadataPath = join(blogRoot, 'crates/services/web/metadata.yml');
  let baseUrl = 'https://tyingshoelaces.com';

  if (process.env.SITEMAP_BASE_URL) {
    baseUrl = process.env.SITEMAP_BASE_URL;
  } else if (apiUrl.includes('localhost') || apiUrl.includes('127.0.0.1')) {
    baseUrl = apiUrl;
  } else if (existsSync(metadataPath)) {
    const metadata = parseYaml(readFileSync(metadataPath, 'utf8'));
    baseUrl = metadata.site?.baseUrl || baseUrl;
  }

  console.log(`📍 Using base URL: ${baseUrl}\n`);

  const routes = [];
  const sourceStats = {};
  const blogItems = [];

  // Process each content source
  for (const [name, source] of Object.entries(contentConfig.content_sources)) {
    if (!source.enabled) {
      console.log(`⏭️  Skipping disabled source: ${name}`);
      continue;
    }

    if (!source.sitemap?.enabled) {
      console.log(`⏭️  Sitemap disabled for source: ${name}`);
      continue;
    }

    console.log(`📂 Processing source: ${name} (${source.sitemap.fetch_from})`);

    const items = await fetchContentItems(source);

    // Track stats
    sourceStats[name] = items.length;

    items.forEach(item => {
      const url = source.sitemap.url_pattern.replace('{slug}', item.slug);
      const routeEntry = {
        loc: `${baseUrl}${url}`,
        lastmod: item.updated_at || item.published_at || today,
        changefreq: source.sitemap.changefreq,
        priority: source.sitemap.priority,
        title: item.title,
        published_at: item.published_at
      };

      routes.push(routeEntry);

      // Track blog items for dynamic lastmod
      if (name === 'blog') {
        blogItems.push(routeEntry);
      }
    });

    console.log(`   ✓ Added ${items.length} URLs`);
  }

  // Find newest dates for dynamic lastmod
  const newestBlogDate = blogItems.length > 0 ? getNewestDate(blogItems) : today;
  const newestDate = getNewestDate(routes);

  // Add static app routes with dynamic lastmod
  routes.push(
    { loc: `${baseUrl}/blog`, lastmod: newestBlogDate, changefreq: 'daily', priority: '0.9' },
    { loc: `${baseUrl}/hire-me`, lastmod: newestDate, changefreq: 'monthly', priority: '0.8' },
    { loc: `${baseUrl}/rss.xml`, lastmod: newestBlogDate, changefreq: 'daily', priority: '0.7' },
    { loc: `${baseUrl}/`, lastmod: newestDate, changefreq: 'weekly', priority: '1.0' }
  );

  console.log(`   ✓ Added 4 static routes (dynamic lastmod)`);

  // Sort by priority (descending) for better crawling
  routes.sort((a, b) => {
    const priorityA = parseFloat(a.priority);
    const priorityB = parseFloat(b.priority);
    return priorityB - priorityA;
  });

  // Write main sitemap
  const distDir = join(__dirname, '../dist');
  const sitemaps = splitSitemaps(routes);
  const sitemapFiles = [];

  if (sitemaps.length === 1) {
    // Single sitemap file
    const xml = generateSitemapXml(routes);
    const sitemapPath = join(distDir, 'sitemap.xml');
    writeFileSync(sitemapPath, xml);
    sitemapFiles.push({
      loc: `${baseUrl}/sitemap.xml`,
      lastmod: today,
      path: sitemapPath,
      size: xml.length
    });
  } else {
    // Multiple sitemap files with index
    sitemaps.forEach((sitemap, idx) => {
      const filename = `sitemap-${sitemap.index}.xml`;
      const xml = generateSitemapXml(sitemap.urls);
      const sitemapPath = join(distDir, filename);
      writeFileSync(sitemapPath, xml);
      sitemapFiles.push({
        loc: `${baseUrl}/${filename}`,
        lastmod: today,
        path: sitemapPath,
        size: xml.length
      });
    });

    // Generate sitemap index
    const indexXml = generateSitemapIndex(sitemapFiles);
    const indexPath = join(distDir, 'sitemap.xml');
    writeFileSync(indexPath, indexXml);
  }

  // Output comprehensive statistics
  console.log(`\n${'='.repeat(60)}`);
  console.log('📊 SITEMAP GENERATION SUMMARY');
  console.log(`${'='.repeat(60)}`);
  console.log(`\n📝 Sources:`);
  for (const [source, count] of Object.entries(sourceStats)) {
    console.log(`   ${source.padEnd(15)} ${count.toString().padStart(5)} URLs`);
  }

  console.log(`\n📂 Generated Files:`);
  sitemapFiles.forEach(file => {
    const sizeKb = (file.size / 1024).toFixed(2);
    console.log(`   ${basename(file.path).padEnd(25)} ${sizeKb.padStart(8)} KB`);

    // Warn if exceeds limits
    if (file.size > MAX_SITEMAP_SIZE_BYTES) {
      console.log(`   ⚠️  WARNING: Exceeds 50MB limit!`);
    }
  });

  console.log(`\n📈 Statistics:`);
  console.log(`   Total URLs: ${routes.length}`);
  if (sitemaps.length > 1) {
    console.log(`   Sitemaps (indexed): ${sitemaps.length}`);
  }

  console.log(`\n🔗 Sitemaps:`);
  sitemapFiles.forEach(file => {
    console.log(`   ${file.loc}`);
  });

  console.log(`\n✅ Sitemap generation complete!\n`);
}

generateSitemap().catch(error => {
  console.error('❌ Sitemap generation failed:', error);
  process.exit(1);
});
