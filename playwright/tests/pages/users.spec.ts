// The roster (/admin/users): every account on the instance, its roles, its
// department and its spend, with the search and role filters the server
// answers.
import { test, expect, apiAs, uniqueEmail, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { UsersPage } from '../support/pages/users.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.users;

test.describe('renders', () => {
  test('lists the seeded people', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(adminPage.locator(SEL.tableRow).filter({ hasText: 'e2e-member-1' })).toHaveCount(1);
  });

  test('names each person\'s department', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('e2e-member-4');
    await expect(adminPage.locator(SEL.tableRow).first()).toContainText('Product');
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('pages at fifty rows', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeLessThanOrEqual(50);
  });
});

test.describe('actions', () => {
  test('search narrows the roster', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('e2e-member-1');
    const rows = adminPage.locator(SEL.tableRow);
    await expect(rows.filter({ hasText: 'e2e-member-1' })).toHaveCount(1);
    await expect(rows.filter({ hasText: 'e2e-member-3' })).toHaveCount(0);
  });

  test('a row opens the person', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('e2e-member-1');
    await adminPage.locator(`${SEL.tableRow} a[href^="/admin/user?id="]`).first().click();
    await expect(adminPage).toHaveURL(/\/admin\/user\?id=e2e-member-1/);
  });

  test('creates a user through the API the page posts to, then removes it', async ({ request }) => {
    const email = uniqueEmail('created');
    const created = await request.post('/api/public/admin/users', {
      ...apiAs(request, 'admin'),
      data: { user_id: email.split('@')[0], display_name: 'E2E Created', email },
    });
    expect([200, 201]).toContain(created.status());
    const search = await request.get(
      `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
      apiAs(request, 'admin'),
    );
    expect(search.ok()).toBeTruthy();
    const found = ((await search.json()) as { users: { id: string }[] }).users;
    expect(found.length).toBeGreaterThan(0);
    const removed = await request.delete(`/api/public/admin/users/${found[0].id}`, apiAs(request, 'admin'));
    expect(removed.ok()).toBeTruthy();
  });

  test('a plain user cannot read the roster API', async ({ request }) => {
    const resp = await request.get('/api/public/admin/users', apiAs(request, 'user'));
    expect([401, 403]).toContain(resp.status());
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
