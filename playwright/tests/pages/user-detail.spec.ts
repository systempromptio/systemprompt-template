// One person's page (/admin/user?id=…): identity, roles, department and the
// sessions and spend attributed to them.
import { test, expect, AUTH, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { UserDetailPage } from '../support/pages/users.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const SUBJECT = 'e2e-member-1';
const PATH = PATHS.user(SUBJECT);

test.describe('renders', () => {
  test('names the person in the header', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toContainText(SUBJECT);
  });

  test('states their group and roles', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.openTab('membership');
    const main = adminPage.locator('main');
    await expect(main).toContainText('Engineering');
    await detail.openTab('identity');
    await expect(adminPage.getByRole('checkbox', { name: 'User', exact: true })).toBeChecked();
  });

  test('links back to the roster', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.goto();
    await expect(detail.breadcrumb().locator(`a[href="${PATHS.users}"]`)).toBeVisible();
  });
});

test.describe('actions', () => {
  test('the usage tab lists the seeded conversations', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.openTab('usage');
    expect(await adminPage.locator(SEL.tableRow).count()).toBeGreaterThan(0);
  });

  test('a conversation row opens its reader', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.openTab('usage');
    const link = adminPage.locator(`${SEL.tableRow} a[href^="/admin/contexts/"]`).first();
    await expect(link).toBeVisible();
    await link.click();
    await expect(adminPage).toHaveURL(/\/admin\/contexts\/.+/);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);

  test('a plain user is bounced from their own page too', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.user });
    const res = await context.request.get(PATHS.user('e2e-user'), { maxRedirects: 0 });
    expect(res.status()).toBe(303);
    await context.close();
  });

  test('an unknown user is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.user('no-such-account'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  designLanguageTests(PATH, { navPath: PATHS.users });

  test('meets the density bar', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, SUBJECT);
    await detail.openTab('usage');
    await expectDensity(adminPage, 'waterfall');
  });
});
