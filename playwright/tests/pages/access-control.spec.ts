// /admin/access-control — governed entities with expandable rule evidence.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { AccessControlPage } from '../support/pages/access-control.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.accessControl;

test.describe('renders', () => {
  test('lists the rules on this instance', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    expect(await page.ledger().rowCount()).toBeGreaterThan(0);
  });

  test('says what each entity defaults to', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const access = await page.columnValues('Default');
    expect(access.length).toBeGreaterThan(0);
    for (const value of access) expect(['open', 'closed']).toContain(value.trim().toLowerCase());
  });

  test('carries the totals an auditor checks first', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });

  test('links to the group rule editors', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await expect(adminPage.getByRole("link", { name: "Group rules", exact: true })).toHaveAttribute("href", "/admin/groups");
  });
});

test.describe('actions', () => {
  test('the band filter retains entities with role rules', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.filterBy('band', 'role');
    await expect(adminPage).toHaveURL(/band=role/);
    const rows = adminPage.locator('tr.sp-ac-entity');
    expect(await rows.count()).toBeGreaterThan(0);
    for (const row of await rows.all()) {
      await row.locator('summary').click();
      await expect(row.locator('.sp-ac-layer-tag').filter({ hasText: /role/i }).first()).toBeVisible();
    }
  });

  test('the entity-kind filter isolates one kind of entity', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const kind = await page.firstFilterValue('entity_kind');
    await page.filterBy('entity_kind', kind);
    const kinds = await adminPage.locator('tr.sp-ac-entity').evaluateAll(rows => rows.map(row => row.getAttribute('data-entity-type')));
    expect(kinds.length).toBeGreaterThan(0);
    expect(new Set(kinds)).toEqual(new Set([kind]));
  });

  test('expanding an entity exposes its rule decisions', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const entity = adminPage.locator('tr.sp-ac-entity').first();
    await entity.locator('summary').click();
    await expect(entity.locator('.sp-ac-rules')).toBeVisible();
    const access = await entity.locator('.sp-ac-rules tbody tr td:nth-child(3)').allInnerTexts();
    expect(access.length).toBeGreaterThan(0);
    for (const value of access) expect(['allow', 'deny']).toContain(value.trim().toLowerCase());
  });

  test('opens the group creation form', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await adminPage.getByRole("button", { name: "+ New group", exact: true }).click();
    await expect(adminPage.getByRole("dialog", { name: "Create group" })).toBeVisible();
    await expect(adminPage.getByRole("textbox", { name: "Identifier", exact: true })).toBeVisible();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when no rule matches', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.search('zzz-no-such-entity-zzz');
    await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    // Grouped entities carry identity, subjects and resolution metadata on
    // multiple lines, like the other stacked activity listings.
    await expectDensity(adminPage, 'stackedList');
  });
});
