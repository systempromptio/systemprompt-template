#!/usr/bin/env node

/**
 * Copies assets from the implementation layer to the core web src directory
 * before build, allowing customization while keeping core clean.
 *
 * This enables the blog to have custom fonts, logos, and images that survive
 * core platform updates.
 */

import { cpSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const IMPL_ASSETS = process.env.SYSTEMPROMPT_WEB_ASSETS_PATH || join(__dirname, '../../../crates/services/web/assets');
const CORE_ASSETS = join(__dirname, '../src/assets');
const PUBLIC_DIR = join(__dirname, '../public');

function copyAssets() {
  console.log('📦 Copying implementation assets...');

  if (!existsSync(IMPL_ASSETS)) {
    console.log('ℹ️  No implementation assets found, skipping');
    return;
  }

  const assetTypes = ['fonts', 'logos', 'images'];
  let copiedCount = 0;

  for (const assetType of assetTypes) {
    const srcDir = join(IMPL_ASSETS, assetType);
    const destDir = join(CORE_ASSETS, assetType);
    // Also copy fonts to public for static serving
    const publicDestDir = join(PUBLIC_DIR, assetType);

    if (!existsSync(srcDir)) {
      continue;
    }

    try {
      // Ensure destination directory exists
      mkdirSync(destDir, { recursive: true });

      // Copy all files (recursive)
      cpSync(srcDir, destDir, { recursive: true, force: true });

      // Also copy fonts to public for static serving with /fonts/ paths
      if (assetType === 'fonts') {
        mkdirSync(publicDestDir, { recursive: true });
        cpSync(srcDir, publicDestDir, { recursive: true, force: true });
        console.log(`  ✅ Copied ${assetType} (to src/assets and public)`);
      } else {
        console.log(`  ✅ Copied ${assetType}`);
      }
      copiedCount++;
    } catch (error) {
      console.error(`  ❌ Failed to copy ${assetType}:`, error.message);
    }
  }

  // Copy favicon if it exists
  const favIconSrc = join(IMPL_ASSETS, 'favicon.ico');
  const favIconDest = join(__dirname, '../public/favicon.ico');
  if (existsSync(favIconSrc)) {
    try {
      mkdirSync(dirname(favIconDest), { recursive: true });
      cpSync(favIconSrc, favIconDest, { force: true });
      console.log(`  ✅ Copied favicon`);
      copiedCount++;
    } catch (error) {
      console.error(`  ❌ Failed to copy favicon:`, error.message);
    }
  }

  // Copy robots.txt if it exists
  const robotsTxtSrc = join(IMPL_ASSETS, 'robots.txt');
  const robotsTxtDest = join(__dirname, '../public/robots.txt');
  if (existsSync(robotsTxtSrc)) {
    try {
      mkdirSync(dirname(robotsTxtDest), { recursive: true });
      cpSync(robotsTxtSrc, robotsTxtDest, { force: true });
      console.log(`  ✅ Copied robots.txt`);
      copiedCount++;
    } catch (error) {
      console.error(`  ❌ Failed to copy robots.txt:`, error.message);
    }
  }

  // Copy llms.txt if it exists
  const llmsTxtSrc = join(IMPL_ASSETS, 'llms.txt');
  const llmsTxtDest = join(__dirname, '../public/llms.txt');
  if (existsSync(llmsTxtSrc)) {
    try {
      mkdirSync(dirname(llmsTxtDest), { recursive: true });
      cpSync(llmsTxtSrc, llmsTxtDest, { force: true });
      console.log(`  ✅ Copied llms.txt`);
      copiedCount++;
    } catch (error) {
      console.error(`  ❌ Failed to copy llms.txt:`, error.message);
    }
  }

  if (copiedCount === 0) {
    console.log('ℹ️  No assets to copy');
  } else {
    console.log(`✨ Copied ${copiedCount} asset type(s) from implementation layer`);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  copyAssets();
}

export { copyAssets };
