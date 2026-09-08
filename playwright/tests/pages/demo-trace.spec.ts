// /admin/demo/trace — the scripted walk through one governed tool call.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { DemoTracePage } from '../support/pages/demo-trace.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.demoTrace;

test.describe('renders', () => {
  test('lays out the walk as ordered steps', async ({ adminPage }) => {
    const page = new DemoTracePage(adminPage);
    await page.goto();
    expect(await page.steps().count()).toBeGreaterThan(1);
  });

  test('names the chain stages it walks through', async ({ adminPage }) => {
    const page = new DemoTracePage(adminPage);
    await page.goto();
    await expect(adminPage.locator('main')).toContainText(/hook|decision|audit/i);
  });
});

test.describe('actions', () => {
  test('links into the real decision log', async ({ adminPage }) => {
    const page = new DemoTracePage(adminPage);
    await page.goto();
    const link = adminPage.locator(`main a[href^="${PATHS.governanceDecisions}"], main a[href^="${PATHS.traces}"]`).first();
    await expect(link).toBeVisible();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('meets the density bar', async ({ adminPage }) => {
    const page = new DemoTracePage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'detail');
  });
});
