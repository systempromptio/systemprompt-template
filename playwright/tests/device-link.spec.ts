import { test, expect } from '@playwright/test';

// The device-link page is the entry point of the developer onboarding flow
// (clean client / dev sandbox paste a code shown here). It must render up to
// the auth boundary.

test('device-link page renders', async ({ page }) => {
  const response = await page.goto('/bridge-auth/device-link');
  expect(response?.status()).toBeLessThan(500);
  await expect(page.locator('body')).toBeVisible();
});

// The unauthenticated bounce must carry the full device-link target in
// ?redirect= — that preserved query is what lets both sign-in paths (the
// Salesforce start link server-side, the passkey flow via resolveRedirect's
// /bridge-auth/ allowance) resume the device link after login.
test('unauthenticated device-link hit preserves its redirect through the login bounce', async ({
  page,
}) => {
  await page.goto(
    '/bridge-auth/device-link?redirect=' +
      encodeURIComponent('http://127.0.0.1:8767/callback'),
  );
  await page.waitForURL(/\/admin\/login/);
  const url = new URL(page.url());
  const redirect = url.searchParams.get('redirect') ?? '';
  expect(redirect).toContain('/bridge-auth/device-link');
  expect(redirect).toContain('127.0.0.1');
});
