// /admin/evals/runs/{id} — one judge run: its verdict mix and every scored
// request. The seed completes one run, so this never has to skip.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { EvalRunDetailPage } from '../support/pages/evals.page';
import { EVAL_RUN_ID } from '../../setup/seed';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.evalRun(EVAL_RUN_ID);

test.describe('renders', () => {
  test('names the run and its judge', async ({ adminPage }) => {
    const detail = new EvalRunDetailPage(adminPage, PATH);
    await detail.goto();
    await expect(adminPage.locator('main')).toContainText('claude-sonnet-5');
  });

  test('lists every scored result with its verdict', async ({ adminPage }) => {
    const detail = new EvalRunDetailPage(adminPage, PATH);
    await detail.goto();
    expect(await detail.table().rowCount()).toBeGreaterThan(0);
    const verdicts = new Set(
      (await detail.verdictBadges().allInnerTexts()).map((v) => v.trim().toLowerCase()),
    );
    expect(verdicts.has('pass'), `verdicts seen: ${[...verdicts].join(', ')}`).toBe(true);
  });

  test('states the verdict totals above the table', async ({ adminPage }) => {
    const detail = new EvalRunDetailPage(adminPage, PATH);
    await detail.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('a result row links to the request it judged', async ({ adminPage }) => {
    const detail = new EvalRunDetailPage(adminPage, PATH);
    await detail.goto();
    const link = adminPage.locator(`${SEL.tableRow} a[href^="/admin/requests/"]`).first();
    await expect(link).toBeVisible();
    await link.click();
    await expect(adminPage).toHaveURL(/\/admin\/requests\/.+/);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);

  test('an unknown run is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.evalRun('no-such-run'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  designLanguageTests(PATH, { navPath: PATHS.evals });

  test('meets the density bar', async ({ adminPage }) => {
    const detail = new EvalRunDetailPage(adminPage, PATH);
    await detail.goto();
    await expectDensity(adminPage, 'detail');
  });
});
