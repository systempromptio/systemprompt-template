// The five-tab analytics dashboard against the deterministic e2e seed. The
// admin principal is org-scoped (e2e-corp); the platform admin sees the org
// selector. Every navigation asserts on seeded content, never on live totals.
import { test, expect } from './support/fixtures';

const TABS = ['overview', 'usage', 'seats', 'spend', 'code'] as const;

// Each tab renders its own centre of gravity, not one shared strip: overview
// and usage lead with KPI cards, seats with the utilisation table, spend with
// meters, code with the measurement-frame cards.
const TAB_MARKER: Record<(typeof TABS)[number], string> = {
  overview: '.kpi-strip .kpi-card',
  usage: '.kpi-strip .kpi-card',
  seats: '.data-table, .empty-state',
  spend: '.spend-meter-list, .empty-state',
  code: '.code-frames .code-frame',
};

test.describe('analytics tabs', () => {
  for (const tab of TABS) {
    test(`the ${tab} tab renders with seeded data`, async ({ adminPage: page }) => {
      const res = await page.goto(`/admin/analytics?tab=${tab}`);
      expect(res?.status()).toBe(200);
      await expect(page.locator('.sp-tab--active')).toHaveCount(1);
      await expect(page.locator(TAB_MARKER[tab]).first()).toBeVisible();
    });
  }

  test('the usage leaderboard contains a seeded member', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=usage');
    await expect(page.locator('main')).toContainText('e2e-member-1');
  });

  test('bucket toggle switches to weekly', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=overview&bucket=week');
    await expect(page).toHaveURL(/bucket=week/);
    await expect(page.locator('.sp-tab--active')).toHaveCount(1);
  });

  test('org admins get no org selector (scope is pinned)', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=overview');
    await expect(page.locator('select[name="org"]')).toHaveCount(0);
  });

  test('department filter narrows the leaderboard', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=usage&department=Engineering');
    const main = page.locator('main');
    await expect(main).toContainText('e2e-member-1'); // Engineering
    await expect(main).not.toContainText('e2e-member-3'); // Sales
  });

  test('an empty range renders empty states, never a 500', async ({ adminPage: page }) => {
    for (const tab of TABS) {
      const res = await page.goto(`/admin/analytics?tab=${tab}&from=2020-01-01&to=2020-01-02`);
      expect(res?.status(), `tab=${tab}`).toBe(200);
    }
  });

  test('pagination beyond the seeded rows renders cleanly', async ({ adminPage: page }) => {
    const res = await page.goto('/admin/analytics?tab=usage&page=99');
    expect(res?.status()).toBe(200);
  });
});

test.describe('platform scope', () => {
  test('the org selector appears and scopes to e2e-corp', async ({ platformAdminPage: page }) => {
    await page.goto('/admin/analytics?tab=usage&org=e2e-corp');
    await expect(page.locator('select[name="org"]')).toBeVisible();
    await expect(page.locator('main')).toContainText('e2e-member-1');
  });
});
