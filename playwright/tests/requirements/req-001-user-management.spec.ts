// REQ-001 — Admin User Management.
//
// Acceptance criteria: "Admin can search/list users, create users, modify
// role/status, disable/delete access, review basic account activity, and revoke
// sessions without CLI or database access."
//
// Every assertion here is about a state CHANGE or a REFUSAL, never about an
// element merely being present. The register exists because a search box that
// did nothing passed a test that only checked the matching row was visible.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-001 admin user management', () => {
  test('@REQ-001 the roster is reachable from the sidebar, not just a breadcrumb', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/analytics');
    const link = page.locator('.admin-sidebar a[href="/admin/access/users"]');
    await expect(link).toBeVisible();
    await link.click();
    await expect(page).toHaveURL(/\/admin\/access\/users/);
  });

  test('@REQ-001 search filters the roster', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    const match = page.locator('tr[data-user-name*="e2e-member-1"]');
    const other = page.locator('tr[data-user-name*="e2e-member-3"]');
    await expect(match).toBeVisible();
    await expect(other).toBeVisible();

    await page.locator('#user-search').fill('e2e-member-1');
    await expect(match).toBeVisible();
    await expect(other).toBeHidden();
  });

  test('@REQ-001 a session can be listed and revoked from the UI', async ({
    platformAdminPage: page,
    request,
  }) => {
    const target = 'e2e-member-1';
    await page.goto(`/admin/access/user?id=${target}`);

    const rows = page.locator('#user-sessions-body tr[data-session-id]');
    await expect(rows.first()).toBeVisible();
    const before = await rows.count();
    expect(before).toBeGreaterThan(0);

    // Revoking must actually move a row to Revoked, not just return 200.
    page.once('dialog', (d) => d.accept());
    await page.locator('[data-revoke-session]').first().click();
    await expect(page.locator('#user-sessions-body .badge-gray').first()).toBeVisible();

    // And the API must agree: at least one session now carries revoked_at.
    const resp = await request.get(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    expect(resp.ok()).toBeTruthy();
    const body = (await resp.json()) as { sessions: { revoked_at: string | null }[] };
    expect(body.sessions.some((s) => s.revoked_at !== null)).toBeTruthy();
  });

  test('@REQ-001 "sign out everywhere" leaves no live session', async ({
    platformAdminPage: page,
    request,
  }) => {
    const target = 'e2e-member-2';
    const resp = await request.delete(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    expect(resp.ok()).toBeTruthy();

    const after = await request.get(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    const body = (await after.json()) as { sessions: { revoked_at: string | null }[] };
    expect(body.sessions.every((s) => s.revoked_at !== null)).toBeTruthy();
  });

  test('@REQ-001 an org admin cannot escalate a user to admin by editing them', async ({
    request,
  }) => {
    // The create and invite paths already refused this; the edit path did not,
    // which made it a way around both.
    const resp = await request.put(
      '/api/public/admin/users/e2e-member-1',
      { ...apiAs(request, 'admin'), data: { roles: ['user', 'admin'] } },
    );
    expect(resp.status()).toBe(403);
  });

  test('@REQ-001 a non-admin reaches none of it', async ({ request }) => {
    const resp = await request.get(
      '/api/public/admin/users/e2e-member-1/sessions',
      apiAs(request, 'user'),
    );
    expect(resp.status()).toBe(403);
  });
});
