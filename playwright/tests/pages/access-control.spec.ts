// /admin/access-control — the rule ledger and the grant editor.
//
// The ledger is server-rendered, one row per `access_control_rules` row, at
// the band it is written at: the entity it governs, the subject it applies
// to, and the access it resolves to.
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

  test('says what each rule resolves to', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const access = await page.columnValues('Access');
    expect(access.length).toBeGreaterThan(0);
    for (const value of access) expect(['allow', 'deny']).toContain(value.trim().toLowerCase());
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
  test('the subject-kind filter narrows the ledger to one band', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.filterBy('subject_kind', 'role');
    const kinds = await page.columnValues('Subject kind');
    for (const kind of kinds) expect(kind.trim()).toBe('role');
  });

  test('the entity-kind filter isolates one kind of entity', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const kind = await page.firstFilterValue('entity_kind');
    await page.filterBy('entity_kind', kind);
    const kinds = new Set((await page.columnValues('Entity kind')).map((k) => k.trim()));
    expect(kinds.size).toBe(1);
  });

  test('sorting by subject kind reorders the ledger', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const direction = await page.ledger().sortBy('Subject kind');
    expect(['ascending', 'descending']).toContain(direction);
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
    await expectDensity(adminPage, 'list');
  });
});
