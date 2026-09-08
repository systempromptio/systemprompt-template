import { test, expect, type Page } from '@playwright/test';
import { resolve } from 'node:path';

// Reuse the existing seed/login storage states. The isolated smoke config does
// not run global setup, so this spec cannot accidentally seed a shared database.
const adminState = process.env.E2E_ADMIN_STORAGE_STATE ?? resolve('.auth/admin.json');
const userState = process.env.E2E_USER_STORAGE_STATE ?? resolve('.auth/user.json');
const paths = [
  '/admin', '/admin/analytics', '/admin/users', '/admin/groups',
  '/admin/projects', '/admin/roles', '/admin/devices', '/admin/contexts',
  '/admin/history', '/admin/mcp', '/admin/marketplaces', '/admin/plugins',
  '/admin/skills', '/admin/gateway', '/admin/profile',
];

function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if (message.type() === 'error') errors.push(message.text());
  });
  return errors;
}

test.describe('dashboard port', () => {
  test.use({ storageState: adminState });

  for (const path of paths) {
    test(`renders ${path} without browser errors`, async ({ page }) => {
      const errors = collectErrors(page);
      const bootstrap = page.waitForResponse(response =>
        new URL(response.url()).pathname === '/admin/auth/me');
      const response = await page.goto(path);
      expect(response?.status()).toBe(200);
      expect(new URL(page.url()).pathname).toBe(path);
      await expect(page.locator('main')).toBeVisible();
      await expect(page.locator('nav[aria-label="Admin navigation"]')).toBeVisible();
      await expect(page.locator('main')).not.toContainText(/Internal Server Error|Failed to render template/);
      expect((await bootstrap).ok(), 'shared shell authenticated bootstrap').toBe(true);
      await page.evaluate(() => new Promise<void>(done => requestAnimationFrame(() => done())));
      expect(errors).toEqual([]);
    });
  }

  test('Overview and people navigation reach their own pages', async ({ page }) => {
    await page.goto('/admin');
    const nav = page.getByRole('navigation', { name: 'Admin navigation' });
    for (const path of ['/admin/groups', '/admin/projects', '/admin/roles', '/admin/devices']) {
      await nav.locator(`a[href="${path}"]`).click();
      await expect(page).toHaveURL(new RegExp(`${path}$`));
      await expect(nav.locator(`a[href="${path}"]`)).toHaveAttribute('aria-current', 'page');
    }
    await nav.locator('a[href="/admin"]').click();
    await expect(page.locator('main')).toContainText('Overview');
  });

  test('connection tabs switch visible instructions and keyboard focus', async ({ page }) => {
    const errors = collectErrors(page);
    await page.goto('/admin/profile');
    const tabs = page.getByRole('tablist', { name: 'Client to connect' });
    for (const [name, id] of [
      ['Claude Code', 'claude-code'],
      ['Claude Desktop', 'claude-desktop'],
      ['OpenCode', 'opencode'],
    ]) {
      await tabs.getByRole('tab', { name, exact: true }).click();
      await expect(tabs.getByRole('tab', { name, exact: true })).toHaveAttribute('aria-selected', 'true');
      await expect(page.locator(`#connect-panel-${id}`)).toBeVisible();
    }
    await tabs.getByRole('tab', { name: 'Claude Code', exact: true }).focus();
    await page.keyboard.press('ArrowRight');
    await expect(tabs.getByRole('tab', { name: 'Claude Desktop', exact: true })).toBeFocused();
    expect(errors).toEqual([]);
  });
});

test('plain user owns account/history and cannot open the directory', async ({ browser }) => {
  const context = await browser.newContext({ storageState: userState });
  try {
    const page = await context.newPage();
    for (const path of ['/admin/profile', '/admin/history']) {
      expect((await page.goto(path))?.status()).toBe(200);
      expect(new URL(page.url()).pathname).toBe(path);
    }
    await page.goto('/admin/users');
    await expect(page).toHaveURL(/\/admin\/profile$/);
    await expect(page.locator('nav a[href="/admin/users"]')).toHaveCount(0);
  } finally {
    await context.close();
  }
});

test('anonymous requests retain the destination login flow', async ({ browser }) => {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await page.goto('/admin/users');
    await expect(page).toHaveURL(/\/admin\/login(?:[?#]|$)/);
    await expect(page.locator('input, button').first()).toBeVisible();
    await expect(page.locator('nav a[href="/admin/users"]')).toHaveCount(0);
  } finally {
    await context.close();
  }
});
