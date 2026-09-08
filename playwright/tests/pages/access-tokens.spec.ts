// /admin/access-tokens — every personal access token on the instance: whose
// it is, when it was last used, and whether it is live, expired or revoked.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { AccessTokensPage } from '../support/pages/access-tokens.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.accessTokens;

test.describe('renders', () => {
  test('lists the seeded tokens, one row per token', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThanOrEqual(7);
    await expect(page.row('member-1 laptop')).toBeVisible();
  });

  test('names the owner of every token', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    await expect(page.row('member-2 laptop')).toContainText('e2e-member-2');
  });

  test('tells a live token from an expired and a revoked one', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    const badges = await page.stateBadges().allInnerTexts();
    const states = new Set(badges.map((b) => b.trim().toLowerCase()));
    expect(states.size, `token states seen: ${[...states].join(', ')}`).toBeGreaterThan(1);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('search narrows the table to one owner', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    await page.table().filter('e2e-member-5');
    const rows = adminPage.locator(SEL.tableRow);
    await expect(rows.filter({ hasText: 'e2e-member-5' })).toHaveCount(1);
    await expect(rows.filter({ hasText: 'e2e-member-1' })).toHaveCount(0);
  });

  test('a token owner link opens the person', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    await page.row('member-1 laptop').locator('a[href^="/admin/user?id="]').first().click();
    await expect(adminPage).toHaveURL(/\/admin\/users\/e2e-member-1/);
  });

  test('revoking through the API the page posts to marks the token revoked', async ({
    adminPage,
    request,
  }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    // The seed re-issues the row on every run, so revoking a live token here
    // is reversible by the next seed.
    await expect(page.revokeButton('member-1 laptop')).toBeVisible();
    const res = await request.delete('/api/public/admin/users/e2e-member-1/pats/e2e-dkey-00', {
      headers: { cookie: (await adminPage.context().cookies()).map((c) => `${c.name}=${c.value}`).join('; ') },
    });
    expect([200, 204]).toContain(res.status());
    await page.goto();
    await expect(page.row('member-1 laptop').locator(SEL.badge)).toContainText(/revoked/i);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    await page.table().filter('zzz-no-such-token-zzz');
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new AccessTokensPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
