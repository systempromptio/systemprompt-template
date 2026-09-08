// The trace list (/admin/traces) and one trace's waterfall.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { TracesPage } from '../support/pages/traces.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.traces;
const WINDOW = { preset: '30d' };
// A custom window nothing was ever seeded into: the page has no free-text
// search, so "nothing matches" is a window rather than a query.
const DEAD_WINDOW = { from: '2001-01-01T00:00:00Z', to: '2001-01-02T00:00:00Z' };

test.describe('renders', () => {
  test('lists the seeded traces, one row per trace id', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    expect(await page.traceLinks().count()).toBeGreaterThan(0);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('the detail page draws the spans as one waterfall', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.spanTable()).toBeVisible();
    expect(await detail.bars().count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('sorting by duration reorders the table', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    const direction = await page.sortBy('Duration');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('the user facet narrows the table', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    await page.filterByUser('e2e-member-1');
    const rows = adminPage.locator(SEL.tableRow);
    expect(await rows.count()).toBeGreaterThan(0);
    await expect(rows.filter({ hasText: 'e2e-member-3' })).toHaveCount(0);
  });

  test('an unknown trace id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.trace('no-such-trace'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH, { query: WINDOW });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(DEAD_WINDOW);
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'list');
  });

  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto(WINDOW);
    await list.openFirst();
    // Why: the waterfall is the page; the span table beneath it is the index,
    // so the row budget is the list shape's chrome checks without a row count.
    await expectDensity(adminPage, 'waterfall');
  });
});
