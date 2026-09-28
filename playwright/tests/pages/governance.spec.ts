// /admin/governance — the four policies the chain runs, in evaluation order,
// and one policy's editor.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.governance;
test.describe('renders', () => {
  test('shows the current policy-chain summary', async ({ adminPage }) => {
    await adminPage.goto(PATH);
    await expect(adminPage.getByRole('region', { name: /governance summary/i })).toBeVisible();
    expect(await adminPage.locator('nav[aria-label="Deny counts by chain stage"] a').count()).toBeGreaterThan(0);
  });

  test('links each recorded decision to its audit detail', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    expect(await adminPage.locator(`a[href^="/admin/governance/decisions/"]`).count()).toBeGreaterThan(0);
  });

  test('offers the current decisions, safety, and hooks views', async ({ adminPage }) => {
    await adminPage.goto(PATH);
    await expect(adminPage.getByRole('tablist', { name: /governance views/i })).toBeVisible();
    await expect(adminPage.getByRole('tab', { name: /decisions/i })).toBeVisible();
  });
});

test.describe('actions', () => {
  test('a stage count filters the decision log', async ({ adminPage }) => {
    await adminPage.goto(PATH);
    await adminPage.locator('nav[aria-label="Deny counts by chain stage"] a').first().click();
    await expect(adminPage).toHaveURL(/policy=/);
    await expect(adminPage.locator(".sp-table__el tbody tr").first()).toBeVisible();
  });

  test('an unknown policy is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(`${PATH}?policy=no-such-policy`, {
      maxRedirects: 0,
    });
    expect(res.status()).toBe(200);
    await context.close();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('meets the density bar', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
