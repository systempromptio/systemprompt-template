// /admin/governance/decisions — the decision log: one row per chain
// decision, with denials at every one of the four stages.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL } from '../support/pages/selectors';
import { GovernanceDecisionsPage } from '../support/pages/governance-decisions.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.governanceDecisions;
const WINDOW = { preset: '30d' };

test.describe('renders', () => {
  test('lists the seeded decisions, one row each', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('shows allow and deny side by side', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    const badges = new Set(
      (await page.decisionBadges().allInnerTexts()).map((b) => b.trim().toLowerCase()),
    );
    expect(badges.has('allow'), `decisions seen: ${[...badges].join(', ')}`).toBe(true);
    expect(badges.has('deny'), `decisions seen: ${[...badges].join(', ')}`).toBe(true);
  });

  test('states the totals above the table', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('the decision facet narrows the log to denials', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto({ ...WINDOW, outcome: 'deny' });
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    const badges = new Set(
      (await page.decisionBadges().allInnerTexts()).map((b) => b.trim().toLowerCase()),
    );
    expect(badges.has('allow')).toBe(false);
  });

  test('the search box narrows on user or tool', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    await page.table().filter('e2e-member-3');
    const rows = adminPage.locator(SEL.tableRow);
    expect(await rows.count()).toBeGreaterThan(0);
    await expect(rows.filter({ hasText: 'e2e-member-4' })).toHaveCount(0);
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH, { query: WINDOW });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    await page.table().filter('zzz-no-such-decision-zzz');
    await page.table().expectEmpty();
  });

  test('meets the density bar', async ({ adminPage }) => {
    const page = new GovernanceDecisionsPage(adminPage);
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'list');
  });
});
