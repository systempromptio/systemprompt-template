// /admin/settings — the account form and the danger zone.
import { test, expect, ACCOUNT_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SettingsPage } from '../support/pages/settings.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.settings;

test.describe('renders', () => {
  test('shows the account form with the signed-in email', async ({ userPage }) => {
    const page = new SettingsPage(userPage);
    await page.goto();
    await expect(page.displayName()).toBeVisible();
    await expect(page.email()).toHaveValue('e2e-user@e2e.local');
  });

  test('offers a timezone', async ({ userPage }) => {
    const page = new SettingsPage(userPage);
    await page.goto();
    await expect(page.timezone()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('the save control submits the form', async ({ userPage }) => {
    const page = new SettingsPage(userPage);
    await page.goto();
    await expect(page.saveButton()).toBeEnabled();
  });

  // Deleting is irreversible, and an accepted delete would remove the
  // principal the rest of the suite signs in as, so this drives the guard:
  // one click must not fire the request.
  test('one click on delete does not delete', async ({ userPage }) => {
    const page = new SettingsPage(userPage);
    await page.goto();
    let deleted = false;
    userPage.on('request', (r) => {
      if (r.method() === 'DELETE') deleted = true;
    });
    await expect(page.deleteAccountButton()).toHaveAttribute('type', 'button');
    await page.deleteAccountButton().click();
    await userPage.waitForTimeout(500);
    expect(deleted, 'delete fired without a confirmation').toBe(false);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, ACCOUNT_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH, { as: 'userPage' });

  test('meets the density bar', async ({ userPage }) => {
    const page = new SettingsPage(userPage);
    await page.goto();
    await expectDensity(userPage, 'detail');
  });
});
