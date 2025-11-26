import type { Plugin } from 'vite';
import { readFileSync } from 'fs';
import { resolve } from 'path';
import YAML from 'js-yaml';

interface MetadataConfig {
  site: {
    name: string;
    title: string;
    description: string;
    author: string;
    baseUrl: string;
  };
  seo: {
    defaultImage: string;
    twitterHandle: string;
    keywords: string;
  };
  copyright: {
    holder: string;
    year: number;
    link: string;
    notice: string;
  };
}

/**
 * Vite plugin that injects metadata from crates/services/web/metadata.yml
 * into the index.html file at build time.
 *
 * Replaces placeholder values in HTML with actual metadata:
 * - {{SITE_TITLE}}
 * - {{SITE_DESCRIPTION}}
 * - {{THEME_COLOR}}
 * - {{FAVICON_PATH}}
 * - {{API_DOMAIN}} - Injects proper API domain for preconnect
 */
export function metadataInjector(): Plugin {
  let metadata: MetadataConfig;

  return {
    name: 'metadata-injector',

    configResolved() {
      // Load metadata from services directory
      const metadataPath = resolve(__dirname, '../../../crates/services/web/metadata.yml');
      try {
        const content = readFileSync(metadataPath, 'utf-8');
        metadata = YAML.load(content) as MetadataConfig;
        console.log('[metadata-injector] Loaded metadata from', metadataPath);
      } catch (error) {
        throw new Error(`Failed to load metadata.yml: ${error}`);
      }
    },

    transformIndexHtml(html: string) {
      if (!metadata) {
        console.warn('[metadata-injector] Metadata not loaded, skipping injection');
        return html;
      }

      // Determine API domain (use env var if available, fallback to baseUrl)
      const apiDomain = process.env.VITE_API_BASE_URL || metadata.site.baseUrl;

      // Replace placeholders in HTML
      const result = html
        .replace('{{SITE_TITLE}}', metadata.site.title)
        .replace('{{SITE_DESCRIPTION}}', metadata.site.description)
        .replace('{{THEME_COLOR}}', '#404040') // Monochrome theme color
        .replace('{{FAVICON_PATH}}', '/favicon.ico')
        .replace(/{{SITE_NAME}}/g, metadata.site.name)
        .replace(/{{API_DOMAIN}}/g, apiDomain)
        // Remove font preload comment - fonts load efficiently from CSS with font-display:swap
        .replace(/\s*<!-- Font preloads[^>]*-->/, '');

      return result;
    },
  };
}
