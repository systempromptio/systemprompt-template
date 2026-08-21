import { defineConfig, devices } from '@playwright/test';

// Runs against an already-running gateway (`just start`); it never boots one —
// this clone's server may be shared with other agents, so global-setup pings
// /health and fails fast instead of starting anything. GATEWAY_URL overrides
// the default local port.
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  retries: Number(process.env.E2E_RETRIES ?? 0),
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  globalSetup: './setup/global-setup.ts',
  snapshotPathTemplate: '{testDir}/__screenshots__/{testFileName}/{arg}{ext}',
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01,
      animations: 'disabled',
      caret: 'hide',
      stylePath: './tests/support/screenshot.css',
    },
  },
  use: {
    baseURL: process.env.GATEWAY_URL ?? 'http://localhost:8080',
    trace: 'retain-on-failure',
    // Determinism knobs for the visual suite; harmless for functional specs.
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    colorScheme: 'light',
    timezoneId: 'UTC',
    locale: 'en-US',
    contextOptions: { reducedMotion: 'reduce' },
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 900 } },
    },
  ],
});
