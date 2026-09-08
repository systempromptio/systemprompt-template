// /admin/governance/hooks — the hook contract a Claude Code plugin wires to
// this instance, with the settings block a developer pastes.
import { test, expect, CONSOLE_ACCESS } from '../support/fixtures';
import { expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { GovernanceHooksPage } from '../support/pages/governance-hooks.page';
import { authorizationTable, designLanguageTests } from '../support/shared';

const PATH = PATHS.governanceHooks;

test.describe('renders', () => {
  test('names the hook events the gateway answers', async ({ adminPage }) => {
    const page = new GovernanceHooksPage(adminPage);
    await page.goto();
    await expect(adminPage.locator('main')).toContainText(/PreToolUse/);
  });

  test('renders the export a developer pastes', async ({ adminPage }) => {
    const page = new GovernanceHooksPage(adminPage);
    await page.goto();
    await expect(page.exportBlock()).toBeVisible();
    await expect(page.exportBlock()).toContainText(/hooks/i);
  });
});

test.describe('actions', () => {
  test('the export names this instance, not a placeholder host', async ({ adminPage }) => {
    const page = new GovernanceHooksPage(adminPage);
    await page.goto();
    const text = await page.exportBlock().innerText();
    expect(text).not.toContain('example.com');
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, CONSOLE_ACCESS);
});

test.describe('design language', () => {
  designLanguageTests(PATH);

  test('meets the density bar', async ({ adminPage }) => {
    const page = new GovernanceHooksPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'detail');
  });
});
