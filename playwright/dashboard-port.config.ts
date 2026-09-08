import { defineConfig, devices } from '@playwright/test';

// Deliberately no webServer or globalSetup: only the explicitly selected,
// disposable runtime and already-created principal storage states are used.
if (!process.env.GATEWAY_URL || !process.env.E2E_ADMIN_STORAGE_STATE || !process.env.E2E_USER_STORAGE_STATE) {
  throw new Error('Set GATEWAY_URL, E2E_ADMIN_STORAGE_STATE and E2E_USER_STORAGE_STATE for the isolated dashboard smoke run');
}
export default defineConfig({
  testDir: './tests',
  testMatch: 'dashboard-port.spec.ts',
  workers: 1,
  retries: 0,
  reporter: [['list']],
  use: {
    ...devices['Desktop Chrome'],
    baseURL: process.env.GATEWAY_URL,
    viewport: { width: 1440, height: 1000 },
    trace: 'retain-on-failure',
  },
});
