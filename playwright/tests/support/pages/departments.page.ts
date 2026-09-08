// The departments list and one department's detail page.
//
// Both are server-rendered listings: the list is one row per department with
// its member count and spend, and the detail page is the roster of the people
// in it. Sorting and filtering are navigations, so every method returns after
// the re-render.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class DepartmentsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.departments);
  }

  /** Every department link in the table, in row order. */
  departmentLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/departments/"]`);
  }

  /** The row naming a department, by its visible name. */
  row(name: string): Locator {
    return this.page.locator(SEL.tableRow).filter({ hasText: name }).first();
  }

  /** Sort by a column, clicking its header LINK rather than the cell: the
   *  shared TableHandle clicks the `th` itself, whose centre is padding on a
   *  wide column, so the navigation never fires. Returns the resulting
   *  aria-sort of that header.
   */
  async sortBy(header: string): Promise<string> {
    const th = () =>
      this.page
        .locator(`${SEL.tableEl} thead th`)
        .filter({ hasText: new RegExp(`^\\s*${header}`, 'i') })
        .first();
    await th().locator('a').first().click();
    await this.page.waitForLoadState('networkidle');
    return (await th().getAttribute('aria-sort')) ?? 'none';
  }

  async open(name: string): Promise<DepartmentDetailPage> {
    await this.row(name).locator('a[href^="/admin/departments/"]').first().click();
    await expect(this.page).toHaveURL(/\/admin\/departments\/.+/);
    return new DepartmentDetailPage(this.page, this.page.url());
  }
}

export class DepartmentDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  /** The people in this department, as their account ids. */
  async memberIds(): Promise<string[]> {
    return this.page
      .locator(`${SEL.tableRow}[data-user-id]`)
      .evaluateAll((rows) => rows.map((row) => (row as HTMLElement).dataset.userId ?? ''));
  }

  memberLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/user?id="]`);
  }
}
