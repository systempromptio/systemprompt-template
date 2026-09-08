// The shape every page spec follows. This file is not run (the chromium
// project ignores `_*.spec.ts`); copy it when adding a page.
//
// Four describe blocks, always these four names, in this order.
// scripts/check-spec-shape.sh fails any tests/pages/*.spec.ts missing one.
//
//   renders          the page draws its own content from the seeded dataset
//   actions          every mutation the page offers, and its visible result
//   authorization    admin, the plain user and anonymous, with literal statuses
//   design language  the shell, the tokens, focus, reflow, axe, and the
//                    measured density bar
//
// The authorization block is a table (support/shared.ts), so a reviewer can
// see at a glance that the matrix is complete. Console pages use
// CONSOLE_ACCESS; the account pages every signed-in person may read use
// ACCOUNT_ACCESS; anything else writes its own rows.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.users;

test.describe('renders', () => {
  test('lists the seeded people', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('filtering narrows the table', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const before = await page.table().rowCount();
    await page.table().filter('e2e-member-1');
    expect(await page.table().rowCount()).toBeLessThan(before);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  // The measured bar. Pass 'detail' on a single-record page; the only
  // difference is how many rows are expected above the fold.
  test('meets the density bar', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
