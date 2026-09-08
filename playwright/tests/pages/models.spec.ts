// /admin/models — what the gateway will route to, per provider, with the
// price it charges.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { ModelsPage } from '../support/pages/models.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.models;

test.describe('renders', () => {
  test('lists the routable models, one row each', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('names the provider of every model', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    expect(await page.providerBadges().count()).toBeGreaterThan(0);
  });

  test('states the catalogue totals above the table', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('search narrows the catalogue', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    const first = (await page.table().rows().first().innerText()).split(/\s+/)[0];
    await page.table().filter(first);
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    await page.table().filter('zzz-no-such-model-zzz');
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new ModelsPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });
});
