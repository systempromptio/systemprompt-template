// The auth fixture's acceptance test: one case per principal proving the
// storageState minted in global-setup lands on the right side of every gate
// (anonymous redirect, non-admin bounce, admin pass, platform gate).
import { test, expect, apiAs } from './support/fixtures';

test.describe('anonymous', () => {
  test('analytics redirects to login carrying the target', async ({ anonPage }) => {
    await anonPage.goto('/admin/analytics');
    await expect(anonPage).toHaveURL(/\/admin\/login\?.*redirect=/);
  });

  test('users roster redirects to login', async ({ anonPage }) => {
    await anonPage.goto('/admin/access/users');
    await expect(anonPage).toHaveURL(/\/admin\/login/);
  });
});

test.describe('authenticated non-admin', () => {
  test('the admin root bounces to the profile page', async ({ userPage }) => {
    await userPage.goto('/admin');
    await expect(userPage).toHaveURL(/\/admin\/profile/);
  });

  test('analytics bounces to the profile page', async ({ userPage }) => {
    await userPage.goto('/admin/analytics');
    await expect(userPage).toHaveURL(/\/admin\/profile/);
  });

  test('the admin API refuses with 401/403', async ({ request }) => {
    const res = await request.get('/api/public/admin/users', apiAs(request, 'user'));
    expect([401, 403]).toContain(res.status());
  });
});

test.describe('org admin', () => {
  test('the admin root lands on analytics', async ({ adminPage }) => {
    await adminPage.goto('/admin');
    await expect(adminPage).toHaveURL(/\/admin\/analytics/);
  });

  test('the enterprise console is platform-only', async ({ adminPage }) => {
    const res = await adminPage.goto('/admin/enterprises');
    expect(res?.status()).toBe(403);
  });
});

test.describe('platform admin', () => {
  test('the enterprise console renders', async ({ platformAdminPage }) => {
    const res = await platformAdminPage.goto('/admin/enterprises');
    expect(res?.status()).toBe(200);
  });
});
