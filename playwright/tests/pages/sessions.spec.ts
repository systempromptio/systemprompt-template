// The sessions list (/admin/sessions) and one session's detail page.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { SessionsPage } from '../support/pages/sessions.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.sessions;
const WINDOW = { preset: '30d' };
// A custom window nothing was ever seeded into: the page has no free-text
// search, so "nothing matches" is a window rather than a query.
const DEAD_WINDOW = { from: '2001-01-01T00:00:00Z', to: '2001-01-02T00:00:00Z' };

test.describe('renders', () => {
  test('lists the seeded sessions, one row per session', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    expect(await page.sessionLinks().count()).toBeGreaterThan(0);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('the detail page lists the requests in the session', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    await expect(detail.breadcrumb()).toBeVisible();
    await adminPage.getByRole('tab', { name: /^Requests/ }).click();
    await expect(detail.requestsTable()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the table and marks the header', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const direction = await page.sortBy('Cost');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('the user facet narrows the table', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const before = await page.table().rowCount();
    await page.filterByUser('e2e-member-2');
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(adminPage.locator(SEL.tableRow).filter({ hasText: 'e2e-member-4' })).toHaveCount(0);
  });

  test('an unknown session id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.session('no-such-session'), { maxRedirects: 0 });
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
    const page = new SessionsPage(adminPage);
    await page.goto(DEAD_WINDOW);
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'stackedList');
  });

  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);
    await list.goto(WINDOW);
    await list.openFirst();
    await expectDensity(adminPage, 'detail');
  });
});
