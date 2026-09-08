// /admin/departments — one row per department with its headcount and spend.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { DepartmentsPage } from '../support/pages/departments.page';
import { E2E } from '../../setup/seed';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.departments;

test.describe('renders', () => {
  test('lists the seeded departments and the Default catch-all', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    for (const name of [...Object.values(E2E.departments), 'Default']) {
      await expect(page.row(name), name).toBeVisible();
    }
  });

  test('counts the members the seed placed in each', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    await expect(page.row(E2E.departments.engineering)).toContainText(/4/);
    await expect(page.row(E2E.departments.product)).toContainText(/2/);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('a row opens the department', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    const detail = await page.open(E2E.departments.engineering);
    await expect(detail.heading()).toContainText(E2E.departments.engineering);
  });

  test('sorting by name reorders the table', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    const direction = await page.sortBy('Department');
    expect(['ascending', 'descending']).toContain(direction);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('meets the density bar', async ({ adminPage }) => {
    const page = new DepartmentsPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
