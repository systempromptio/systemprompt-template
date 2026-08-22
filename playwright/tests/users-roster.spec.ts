// Users roster: department grouping, search, the create-user panel, and the
// per-row actions popup. Users created here use unique @e2e.local emails and
// are deleted through the admin API in the same test.
import { test, expect, apiAs, uniqueEmail } from './support/fixtures';

test.describe('users roster', () => {
  test('groups seeded users by department', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    const engineering = page.locator('tbody.table-group[data-department="Engineering"]');
    await expect(engineering).toContainText('e2e-member-1');
    const sales = page.locator('tbody.table-group[data-department="Sales"]');
    await expect(sales).toContainText('e2e-member-3');
  });

  // Why this asserts on what DISAPPEARS: the previous version filled the box
  // and checked the matching row was still visible, which passed whether or not
  // the box was wired to anything at all -- and it wasn't.
  test('search narrows the roster', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    const match = page.locator('tr[data-user-name*="e2e-member-1"]');
    const other = page.locator('tr[data-user-name*="e2e-member-3"]');
    await expect(match).toBeVisible();
    await expect(other).toBeVisible();

    await page.locator('#user-search').fill('e2e-member-1');
    await expect(match).toBeVisible();
    await expect(other).toBeHidden();
    // The department that holds only non-matching members collapses with them.
    await expect(page.locator('tbody.table-group[data-department="Sales"]')).toBeHidden();

    await page.locator('#user-search').fill('');
    await expect(other).toBeVisible();
  });

  test('a search matching nobody shows the empty state', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    await page.locator('#user-search').fill('zzz-no-such-user');
    await expect(page.locator('#user-search-empty')).toBeVisible();
    await expect(page.locator('tr[data-user-name]:visible')).toHaveCount(0);
  });

  test('create-user panel creates a user with a department', async ({ adminPage: page, request }) => {
    const email = uniqueEmail('created');
    await page.goto('/admin/access/users');
    await page.locator('#btn-add-user').click();

    const panel = page.locator('#create-user-panel, .side-panel, [data-panel="create-user"]').first();
    await expect(panel).toBeVisible();
    await panel.locator('input[name="user_id"], #new-user-id').first().fill(email.split('@')[0]);
    await panel.locator('input[name="email"], #new-user-email').first().fill(email);
    await panel.locator('input[name="display_name"], #new-user-name').first().fill('E2E Created');

    // Department select is populated live from /management/departments (B1).
    const dept = panel.locator('select#new-user-dept, select[name="department"]').first();
    if (await dept.count()) {
      await expect(dept.locator('option', { hasText: 'Engineering' })).toHaveCount(1);
      await dept.selectOption({ label: 'Engineering' });
    }

    await panel.locator('button[type="submit"], .btn-primary').last().click();
    await expect(page.locator('main')).toContainText(email.split('@')[0]);

    // Cleanup through the API so the roster stays deterministic.
    const list = await request.get(
      `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
      apiAs(request, 'admin'),
    );
    if (list.ok()) {
      const found = (await list.json()) as { user_id?: string; id?: string }[];
      const id = found[0]?.user_id ?? found[0]?.id;
      if (id) await request.delete(`/api/public/admin/users/${id}`, apiAs(request, 'admin'));
    }
  });

  test('the per-row actions popup opens', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    await page.locator('button.btn-actions-trigger[data-user-id="e2e-member-1"]').click();
    await expect(page.locator('#user-actions-popup')).toBeVisible();
    await expect(page.locator('#user-actions-popup')).toContainText('Edit User');
  });
});
