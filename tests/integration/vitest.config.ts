import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 60000,
    hookTimeout: 60000,
    teardownTimeout: 60000,
    isolation: true,
    threads: false,
    env: {
      BASE_URL: process.env.BASE_URL || 'http://localhost:8080',
      ADMIN_TOKEN: process.env.ADMIN_TOKEN || '',
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'dist/'],
    },
    setupFiles: ['./vitest-setup.ts'],
  },
  resolve: {
    alias: {
      '@test/shared': path.resolve(__dirname, './shared'),
      '@test/types': path.resolve(__dirname, './shared/types'),
      '@test/utils': path.resolve(__dirname, './shared/utils'),
      '@test/db': path.resolve(__dirname, './shared/utils/db'),
      '@test/config': path.resolve(__dirname, './shared/config.ts'),
    },
  },
});
