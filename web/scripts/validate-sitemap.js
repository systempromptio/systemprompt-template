import { readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { parseStringPromise } from 'xml2js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function validateSitemap() {
  const distDir = join(__dirname, '../dist');
  const sitemapPath = join(distDir, 'sitemap.xml');

  if (!existsSync(sitemapPath)) {
    console.error('❌ Sitemap not found:', sitemapPath);
    console.error('   Run "npm run sitemap:generate" first');
    process.exit(1);
  }

  console.log('📋 Validating sitemap.xml...\n');

  const sitemapXml = readFileSync(sitemapPath, 'utf8');
  const sitemap = await parseStringPromise(sitemapXml);

  const urls = sitemap.urlset.url.map(entry => entry.loc[0]);

  let totalUrls = 0;
  let validUrls = 0;
  let missingUrls = 0;
  const errors = [];

  for (const url of urls) {
    totalUrls++;

    // Extract path from URL (remove domain)
    const urlObj = new URL(url);
    const path = urlObj.pathname;

    // Skip root and special pages (they're handled by React router)
    if (path === '/' || path === '/blog' || path === '/hire-me' || path.endsWith('.xml')) {
      validUrls++;
      continue;
    }

    // Check if prerendered file exists
    // Prerendered files are at /path/to/page/index.html
    const htmlPath = join(distDir, path, 'index.html');

    if (existsSync(htmlPath)) {
      validUrls++;
      console.log(`✅ ${path}`);
    } else {
      missingUrls++;
      errors.push({ url, path, expectedFile: htmlPath });
      console.log(`❌ ${path} (missing: ${path}/index.html)`);
    }
  }

  console.log(`\n${'='.repeat(60)}`);
  console.log('VALIDATION SUMMARY');
  console.log('='.repeat(60));
  console.log(`Total URLs:    ${totalUrls}`);
  console.log(`Valid URLs:    ${validUrls} ✅`);
  console.log(`Missing URLs:  ${missingUrls} ❌`);
  console.log('='.repeat(60));

  if (missingUrls > 0) {
    console.error('\n⚠️  VALIDATION FAILED\n');
    console.error('The following URLs in sitemap.xml do not have corresponding files:\n');

    errors.forEach(({ url, path, expectedFile }) => {
      console.error(`  URL:      ${url}`);
      console.error(`  Path:     ${path}`);
      console.error(`  Expected: ${expectedFile}`);
      console.error('');
    });

    console.error('This typically means:');
    console.error('  1. Sitemap generation is using filenames instead of frontmatter slugs');
    console.error('  2. Prerendering failed or was not run');
    console.error('  3. There is a mismatch between sitemap and prerender scripts');
    console.error('\nRecommended fixes:');
    console.error('  1. Ensure sitemap generation reads frontmatter slugs');
    console.error('  2. Run: npm run content:prerender');
    console.error('  3. Check that markdown frontmatter has correct "slug" field');

    process.exit(1);
  }

  console.log('\n✅ All sitemap URLs are valid!\n');
}

validateSitemap().catch(error => {
  console.error('❌ Validation error:', error);
  process.exit(1);
});
