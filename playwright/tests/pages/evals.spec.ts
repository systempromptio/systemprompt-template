// /admin/evals — the quality dashboard over the seeded traffic and the judge
// run the seed completed over it. Also the console's front door: /admin 308s
// here.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { EvalsPage } from '../support/pages/evals.page';
import { EVAL_RUN_ID } from '../../setup/seed';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.evals;
const WINDOW = { preset: '30d' };

test.describe('renders', () => {
  test('is where /admin lands', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.root, { maxRedirects: 0 });
    expect(res.status()).toBe(308);
    expect(res.headers().location).toBe(PATH);
    await context.close();
  });

  test('states the traffic totals over the window', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.goto(WINDOW);
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('the judge tab lists the seeded run', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.openTab('judge');
    await expect(page.runLinks().filter({ hasText: EVAL_RUN_ID })).not.toHaveCount(0);
  });
});

test.describe('actions', () => {
  test('the tabs switch the body and mark themselves current', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.openTab('traffic');
    await expect(page.activeTab()).toContainText(/traffic/i);
    await page.openTab('judge');
    await expect(page.activeTab()).toContainText(/judge/i);
  });

  test('a run row opens the run', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.openTab('judge');
    const detail = await page.openFirstRun();
    await expect(detail.breadcrumb()).toBeVisible();
  });

  test('the run launcher is a form that posts to the run route', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.openTab('judge');
    await expect(page.runForm()).toHaveAttribute('method', /post/i);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH, { query: WINDOW });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new EvalsPage(adminPage);
    await page.openTab('judge');
    await expectDensity(adminPage, 'list');
  });
});
