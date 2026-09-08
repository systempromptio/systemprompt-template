// /admin/governance — the four policies the chain runs, in evaluation order,
// and one policy's editor.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { GovernancePage } from '../support/pages/governance.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.governance;
// services/governance/config.yaml, by id.
const POLICIES = ['secret_scan', 'scope_check', 'tool_blocklist', 'rate_limit'];

test.describe('renders', () => {
  test('lists every declared policy', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    for (const id of POLICIES) await expect(page.row(id), id).toBeVisible();
  });

  test('links each policy to its editor', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    expect(await page.policyLinks().count()).toBeGreaterThanOrEqual(POLICIES.length);
  });

  test('the editor names the policy and offers the toggle', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    const editor = await page.open('tool_blocklist');
    await expect(editor.heading()).toContainText(/tool_blocklist|blocklist/i);
    await expect(editor.toggleForm()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('a policy row opens its editor', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    await page.row('rate_limit').locator('a[href^="/admin/governance/policies/"]').first().click();
    await expect(adminPage).toHaveURL(/\/admin\/governance\/policies\/rate_limit/);
  });

  test('an unknown policy is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.governancePolicy('no-such-policy'), {
      maxRedirects: 0,
    });
    expect(res.status()).toBe(404);
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
