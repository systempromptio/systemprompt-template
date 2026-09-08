// The contexts list (/admin/contexts) and one context's detail page.
//
// A context is the persisted state of one conversation. The list is what an
// operator scans to find a conversation worth reading; the detail page is the
// transcript plus what the session around it actually touched.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { ContextsPage } from '../support/pages/contexts.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.contexts;

test.describe('renders', () => {
  test('lists the seeded contexts, one row per context id', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    expect(await page.contextLinks().count()).toBeGreaterThan(0);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('the detail page carries breadcrumbs back to the list', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);
    await list.goto();
    const detail = await list.openFirst();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.breadcrumb().getByRole('link', { name: /conversations/i })).toBeVisible();
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the table and marks the header', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const direction = await page.sortBy('Cost');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('the search box narrows the table', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.search('e2e-member-2');
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('an unknown context id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.context('not-a-context'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto({ q: 'zzz-no-such-context-zzz' });
    await expect(page.emptyState()).toBeVisible();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });

  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);
    await list.goto();
    await list.openFirst();
    await expectDensity(adminPage, 'detail');
  });
});
