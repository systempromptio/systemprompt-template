import type { Plugin } from 'vite';
import { existsSync } from 'fs';
import { join, resolve } from 'path';

/**
 * Vite plugin that implements implementation-layer-first content and asset resolution.
 *
 * Allows the implementation layer to override core files by placing files with
 * matching paths in /crates/services/web/. Falls back to core if not found.
 *
 * Pattern:
 * 1. Import: `import content from '@/content/pages/about.md?raw'`
 * 2. Check: `/crates/services/web/content/pages/about.md`
 * 3. Fallback: `/core/web/src/content/pages/about.md`
 */
export function implementationOverlay(): Plugin {
  const coreRoot = resolve(__dirname, '../src');
  const implRoot = process.env.SYSTEMPROMPT_WEB_IMPL_PATH || resolve(__dirname, '../../../crates/services/web');

  return {
    name: 'implementation-overlay',

    resolveId(source: string) {
      // Only handle @ imports for content and assets
      if (!source.startsWith('@/content') && !source.startsWith('@/assets')) {
        return null;
      }

      // Remove @/ prefix and ?raw suffix if present
      let cleanPath = source.replace('@/', '');
      const hasRawQuery = cleanPath.includes('?raw');
      if (hasRawQuery) {
        cleanPath = cleanPath.replace('?raw', '');
      }

      // Try implementation layer first
      const implPath = join(implRoot, cleanPath);
      if (existsSync(implPath)) {
        console.log(`[overlay] Using implementation: ${cleanPath}`);
        return {
          id: hasRawQuery ? `${implPath}?raw` : implPath,
        };
      }

      // Fallback to core
      const corePath = join(coreRoot, cleanPath);
      if (existsSync(corePath)) {
        console.log(`[overlay] Using core fallback: ${cleanPath}`);
        return {
          id: hasRawQuery ? `${corePath}?raw` : corePath,
        };
      }

      // File not found in either location - let Vite handle the error
      return null;
    },
  };
}
