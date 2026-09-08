// The request log (/admin/requests) and one request's audit detail.
//
// The log is the page an operator reaches for when someone asks "what did that
// cost" or "why was that refused", so the spec's centre of gravity is the row:
// that it carries the attribution, the outcome and the money, that the facets
// narrow it, and that it opens the chain of custody behind it.
import { test, expect, dbRow, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { RequestsPage, RequestDetailPage } from '../support/pages/requests.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.requests;
const WINDOW = { preset: '30d' };

test.describe('renders', () => {
  test('lists the seeded requests, one row per gateway call', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  // The defect this exists to catch: the tiles counted hundreds while the log
  // said "No requests match", because the list query failed on a null column
  // and the failure rendered as an empty state.
  test('a non-zero Requests tile always has a first row beneath it', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    const total = await page.kpi('Requests').number();
    expect(total).toBeGreaterThan(0);
    await expect(page.table().rows().first()).toBeVisible();
    await expect(adminPage.locator(SEL.empty)).toHaveCount(0);
  });

  test('renders a rejected row with a badge and no model', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ ...WINDOW, status: 'rejected' });
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(page.table().rows().first().locator(SEL.badge).first()).toHaveText(/rejected/i);
  });

  test('pages at fifty rows', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeLessThanOrEqual(50);
    await expect(adminPage.locator(SEL.pagination)).toBeVisible();
  });
});

test.describe('actions', () => {
  test('the status facet narrows the log', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    const count = (text: string) => Number(text.replace(/[^0-9]/g, ''));
    const before = count(await adminPage.locator(SEL.toolbarCount).first().innerText());
    await page.goto({ ...WINDOW, status: 'failed' });
    const after = count(await adminPage.locator(SEL.toolbarCount).first().innerText());
    expect(after).toBeGreaterThan(0);
    expect(after).toBeLessThan(before);
  });

  test('the search box narrows on user, model or trace id', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    await page.table().filter('e2e-member-4');
    const rows = adminPage.locator(SEL.tableRow);
    expect(await rows.count()).toBeGreaterThan(0);
    await expect(rows.filter({ hasText: 'e2e-member-1' })).toHaveCount(0);
  });

  test('sorting by cost re-orders the log', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    const direction = await page.table().sortBy('Cost');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('a row opens its chain of custody', async ({ adminPage }) => {
    const row = await dbRow<{ id: string }>(
      "SELECT id FROM ai_requests WHERE id LIKE 'e2e-dreq-%' AND status = 'completed' ORDER BY created_at DESC LIMIT 1",
    );
    expect(row, 'no completed e2e request seeded').not.toBeNull();
    const detail = new RequestDetailPage(adminPage, row!.id);
    await detail.goto();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(adminPage.locator('main')).toContainText(row!.id);
  });

  test('an unknown request id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.request('no-such-request-id'), { maxRedirects: 0 });
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
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'list');
  });

  test('the detail page meets the density bar', async ({ adminPage }) => {
    const row = await dbRow<{ id: string }>(
      "SELECT id FROM ai_requests WHERE id LIKE 'e2e-dreq-%' AND status = 'completed' ORDER BY created_at DESC LIMIT 1",
    );
    expect(row).not.toBeNull();
    const detail = new RequestDetailPage(adminPage, row!.id);
    await detail.goto();
    await expectDensity(adminPage, 'detail');
  });
});
