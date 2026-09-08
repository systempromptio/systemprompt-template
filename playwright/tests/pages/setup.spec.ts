// /admin/setup — the onboarding walk every signed-in person may read.
import { test, expect, ACCOUNT_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SetupPage } from '../support/pages/setup.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.setup;

test.describe('renders', () => {
  test('lists the phases in order', async ({ userPage }) => {
    const page = new SetupPage(userPage);
    await page.goto();
    expect(await page.phases().count()).toBeGreaterThanOrEqual(3);
  });

  test('marks at most one phase as the one to do next', async ({ userPage }) => {
    const page = new SetupPage(userPage);
    await page.goto();
    expect(await page.currentPhase().count()).toBeLessThanOrEqual(1);
  });
});

test.describe('actions', () => {
  test('every phase links somewhere', async ({ userPage }) => {
    const page = new SetupPage(userPage);
    await page.goto();
    expect(await page.guideLinks().count()).toBeGreaterThan(0);
  });

  test('sends the reader to their profile', async ({ userPage }) => {
    const page = new SetupPage(userPage);
    await page.goto();
    await userPage.locator(`main a[href="${PATHS.profile}"]`).first().click();
    await expect(userPage).toHaveURL(new RegExp(`${PATHS.profile}$`));
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, ACCOUNT_ACCESS);
});

test.describe('design language', () => {
  // The sidebar files setup under Settings.
  designLanguageTests(PATH, { as: 'userPage', navPath: PATHS.settings });

  test('meets the density bar', async ({ userPage }) => {
    const page = new SetupPage(userPage);
    await page.goto();
    await expectDensity(userPage, 'detail');
  });
});
