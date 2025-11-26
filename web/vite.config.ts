/**
 * Vite Build Configuration
 *
 * Optimized build and development setup for the SystemPrompt web application.
 * Includes React integration, bundle optimization, development proxy, and
 * custom plugins for theme management and metadata injection.
 *
 * Key Features:
 * - Production bundle minification with Terser (removes console & debugger)
 * - CSS minification with Lightning CSS
 * - Code splitting: vendor chunks by library dependency
 * - Development proxy to backend API (http://localhost:8080)
 * - Theme watching and metadata injection plugins
 * - Source maps enabled for production debugging
 *
 * Build Targets:
 * - ES2020 JavaScript (supports all modern browsers)
 * - Tree shaking enabled for unused code removal
 * - Chunk size warning threshold: 1200 KB
 *
 * Environment Variables:
 * - VITE_API_BASE_URL: Production API base URL (optional, defaults to window.location.origin)
 * - VITE_LOG_LEVEL: Logging level (debug|info|warn|error|none)
 * - VITE_LOG_MODULES: Comma-separated module names for filtering
 *
 * Development:
 * - Port: 5173 (default Vite dev server)
 * - API Proxy: /api → http://localhost:8080
 *
 * Production Build:
 * - Command: npm run build
 * - Output: dist/ directory
 * - All console statements and debugger removed
 *
 * @see https://vite.dev/config/
 */

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { themeWatcher } from './plugins/theme-watcher'
import { implementationOverlay } from './plugins/implementation-overlay'
import { metadataInjector } from './plugins/metadata-injector'

export default defineConfig({
  plugins: [
    react(),
    themeWatcher(),
    implementationOverlay(),
    metadataInjector()
  ],
  resolve: {
    alias: {
      '@': '/src',
    },
  },
  optimizeDeps: {
    include: ['react', 'react-dom', 'react/jsx-runtime'],
    esbuildOptions: {
      target: 'es2020',
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    chunkSizeWarningLimit: 1200,
    target: 'es2020',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
      mangle: true,
    } as any,
    cssMinify: 'lightningcss',
    cssCodeSplit: true,
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (id.includes('node_modules')) {
            // Keep original splitting strategy - don't force everything into vendor
            if (id.includes('lucide-react')) {
              return 'vendor-icons';
            }
            if (id.includes('@a2a-js/sdk')) {
              return 'vendor-a2a';
            }
            if (id.includes('recharts')) {
              return 'vendor-charts';
            }
            if (id.includes('react-markdown') || id.includes('remark')) {
              return 'vendor-markdown';
            }
            if (id.includes('@modelcontextprotocol')) {
              return 'vendor-mcp';
            }
            // Don't add 'return vendor' - let Vite handle the rest
          }
        },
      },
    },
    modulePreload: {
      polyfill: false,
    },
  },
  esbuild: {
    drop: ['debugger'],
  },
  server: {
    fs: {
      // Allow serving files from parent directories (for blog content)
      allow: ['..', '../..'],
    },
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  preview: {
    port: 4173,
    strictPort: true,
  },
})
