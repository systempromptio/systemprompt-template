// Org-membership management on the user-detail page. Serial: both tests move
// the same seeded victim user, and the seed re-homes it to e2e-corp each run.
import { test, expect, apiAs } from './support/fixtures';

test.describe.configure({ mode: 'serial' });

test.describe('platform admin', () => {
  test('moves a user to another org and back', async ({ platformAdminPage: page }) => {
    await page.goto('/admin/access/user?id=e2e-victim');
    const widget = page.locator('#org-membership');
    await expect(widget).toBeVisible();

    const orgSelect = page.locator('#org-membership-org');
    await expect(orgSelect.locator('option', { hasText: 'E2E Corp B' })).toHaveCount(1);
    await orgSelect.selectOption('e2e-corp-b');
    await page.locator('#org-membership-role').selectOption('member');
    await page.locator('#org-membership-save').click();
    await expect(page.locator('#org-membership-status')).not.toBeEmpty();

    await page.reload();
    await expect(page.locator('#org-membership-org')).toHaveValue(/e2e-corp-b/i);

    // Move back so the spec is rerunnable even without a reseed.
    await page.locator('#org-membership-org').selectOption('e2e-corp');
    await page.locator('#org-membership-role').selectOption('member');
    await page.locator('#org-membership-save').click();
    await expect(page.locator('#org-membership-status')).not.toBeEmpty();
  });
});

test.describe('org admin (non-platform)', () => {
  test('cannot move users across organizations', async ({ request }) => {
    const res = await request.put('/api/public/admin/management/users/e2e-victim/organization', {
      ...apiAs(request, 'admin'),
      data: { org: 'e2e-corp-b', org_role: 'member' },
    });
    expect([403, 404]).toContain(res.status());
  });
});
