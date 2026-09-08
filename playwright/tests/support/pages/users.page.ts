// The roster and the user detail page.
//
// The roster's filter chips, its role select and the detail page's tabs are all
// links or GET forms the server answers, so every method here is a navigation
// and the assertions belong to the spec. The detail page is addressed on its
// query form (`/admin/user?id=`), which is the route this instance mounts.
import { expect, type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';
import { SEL } from './selectors';

export class UsersPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.users);
  }

  /** The filter chip row above the table. */
  chips(): Locator {
    return this.page.locator('nav[aria-label="Roster filters"] a');
  }

  chip(label: string): Locator {
    return this.chips().filter({ hasText: new RegExp(`^${label}\\b`) }).first();
  }

  async applyChip(label: string): Promise<void> {
    await this.chip(label).click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The chip the server marked current, by its label alone. */
  async activeChip(): Promise<string> {
    const text = await this.page.locator('nav[aria-label="Roster filters"] a[aria-current="true"]')
      .first()
      .innerText();
    return text.trim().split(/\s+/)[0] ?? '';
  }

  async filterByRole(role: string): Promise<void> {
    await this.page.locator('select#user-role').selectOption(role);
    await this.page.locator('form[role="search"] button[type="submit"]').click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The account ids the current page of the roster shows, in row order. */
  async userIds(): Promise<string[]> {
    return this.page.locator(`${SEL.tableEl} tbody tr[data-user-id]`).evaluateAll((rows) =>
      rows.map((row) => (row as HTMLElement).dataset.userId ?? ''),
    );
  }

  /** Sort by a column, clicking its header LINK rather than the cell.
   *
   *  The shared TableHandle clicks the `th` itself, whose centre is padding on
   *  a wide column, so the navigation never fires and the column reports back
   *  as unsorted.
   */
  async sortBy(header: string): Promise<string> {
    const th = this.page
      .locator(`${SEL.tableEl} thead th`)
      .filter({ hasText: new RegExp(`^\\s*${header}`, 'i') })
      .first();
    await th.locator('a').first().click();
    await this.page.waitForLoadState('networkidle');
    const sorted = this.page
      .locator(`${SEL.tableEl} thead th`)
      .filter({ hasText: new RegExp(`^\\s*${header}`, 'i') })
      .first();
    return (await sorted.getAttribute('aria-sort')) ?? 'none';
  }

  async openUser(userId: string): Promise<UserDetailPage> {
    await this.page.locator(`tr[data-user-id="${userId}"] a`).first().click();
    await expect(this.page.locator('h1').first()).toBeVisible();
    return new UserDetailPage(this.page, userId);
  }

  rowCheckbox(userId: string): Locator {
    return this.page.locator(`[data-select-user="${userId}"]`);
  }

  bulkRolesButton(): Locator {
    return this.page.getByRole('button', { name: /change roles/i });
  }

  createButton(): Locator {
    return this.page.getByRole('button', { name: /new user/i });
  }
}

export class UserDetailPage extends BasePage {
  constructor(page: Page, readonly userId: string) {
    super(page, PATHS.user(userId));
  }

  async openTab(slug: string): Promise<void> {
    await this.page.goto(`${PATHS.user(this.userId)}?tab=${slug}`);
    await expect(this.page.locator('h1').first()).toBeVisible();
  }

  activeTab(): Locator {
    return this.page.locator(`${SEL.tab}[aria-selected="true"]`);
  }

  form(name: string): Locator {
    return this.page.locator(`form[data-form="${name}"]`);
  }

  status(name: string): Locator {
    return this.page.locator(`[data-status="${name}"]`);
  }

  roleCheckbox(role: string): Locator {
    return this.page.locator(`form[data-form="user-roles"] input[value="${role}"]`);
  }

  departmentSelect(): Locator {
    return this.page.locator('select[name="department"]');
  }

  async saveRoles(): Promise<void> {
    await this.form('user-roles').getByRole('button', { name: /save roles/i }).click();
    await expect(this.status('user-roles')).toHaveText(/saved/i, { timeout: 10_000 });
  }

}
