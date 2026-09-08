// /admin/profile — the signed-in person's own account page.
//
// Every signed-in principal reaches it, including the one every console page
// bounces: it is the landing spot a non-admin user is sent to.
import { test, expect, ACCOUNT_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { ProfilePage } from '../support/pages/profile.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.profile;

test.describe('renders', () => {
  test('names the signed-in person', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expect(userPage.locator('main')).toContainText('e2e-user');
  });

  test('states the identity facts a support call asks for', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expect(userPage.locator('main')).toContainText(/user id/i);
  });

  test('a non-admin sees only the account section of the sidebar', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expect(page.nav().locator(`a[href="${PATHS.users}"]`)).toHaveCount(0);
    await expect(page.nav().locator(`a[href="${PATHS.settings}"]`)).toHaveCount(1);
  });
});

test.describe('actions', () => {
  test('the header links reach the setup guide', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expect(userPage.locator(`main a[href="${PATHS.setup}"]`).first()).toBeVisible();
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, ACCOUNT_ACCESS);

  test('a plain user is bounced here from every console page', async ({ userPage }) => {
    await userPage.goto(PATHS.users);
    await expect(userPage).toHaveURL(new RegExp(`${PATH}$`));
  });
});

test.describe('design language', () => {
  designLanguageTests(PATH, { as: 'userPage' });

  test('an admin gets the same shell', async ({ adminPage }) => {
    const page = new ProfilePage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
  });

  test('meets the density bar', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expectDensity(userPage, 'form');
  });
});
