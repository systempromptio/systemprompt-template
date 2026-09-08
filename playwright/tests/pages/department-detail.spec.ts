// /admin/departments/{id} — the people in one department and what they spent.
// The id is generated, so the spec resolves it from the seeded name.
import { test, expect, AUTH, CONSOLE_ACCESS, dbRow } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { DepartmentDetailPage } from '../support/pages/departments.page';
import { E2E } from '../../setup/seed';
import { authorizationTable, designLanguageTests } from '../support/shared';

let PATH = '';

test.beforeAll(async () => {
  const row = await dbRow<{ id: string }>('SELECT id FROM departments WHERE name = $1', [
    E2E.departments.engineering,
  ]);
  if (!row) throw new Error('the Engineering department is not seeded');
  PATH = PATHS.department(row.id);
});

test.describe('renders', () => {
  test('names the department in the header', async ({ adminPage }) => {
    const detail = new DepartmentDetailPage(adminPage, PATH);
    await detail.goto();
    await expect(detail.heading()).toContainText(E2E.departments.engineering);
  });

  test('lists exactly the seeded members', async ({ adminPage }) => {
    const detail = new DepartmentDetailPage(adminPage, PATH);
    await detail.goto();
    const rows = adminPage.locator(SEL.tableRow);
    for (const id of ['e2e-admin', 'e2e-member-1', 'e2e-member-2', 'e2e-member-3']) {
      await expect(rows.filter({ hasText: id }), id).toHaveCount(1);
    }
    await expect(rows.filter({ hasText: 'e2e-member-4' })).toHaveCount(0);
  });

  test('links back to the departments list', async ({ adminPage }) => {
    const detail = new DepartmentDetailPage(adminPage, PATH);
    await detail.goto();
    await expect(detail.breadcrumb().locator(`a[href="${PATHS.departments}"]`)).toBeVisible();
  });
});

test.describe('actions', () => {
  test('a member row opens the person', async ({ adminPage }) => {
    const detail = new DepartmentDetailPage(adminPage, PATH);
    await detail.goto();
    await detail.memberLinks().first().click();
    await expect(adminPage).toHaveURL(/\/admin\/user\?id=/);
  });
});

test.describe('authorization', () => {
  // The path is resolved in beforeAll, so the table reads it lazily.
  authorizationTable(() => PATH, CONSOLE_ACCESS);

  test('an unknown department is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.department('no-such-department'), {
      maxRedirects: 0,
    });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  designLanguageTests(() => PATH, { navPath: PATHS.departments });

  test('meets the density bar', async ({ adminPage }) => {
    const detail = new DepartmentDetailPage(adminPage, PATH);
    await detail.goto();
    await expectDensity(adminPage, 'detail');
  });
});
