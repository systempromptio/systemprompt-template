// The access-control page: the rule ledger above, the three-pane editor below.
//
// The ledger is server-rendered and the editor is not, so the two need
// different waits — a ledger filter is a form submission and a tree selection
// is a click that swaps a client-rendered pane.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { TableHandle } from './table';

export class AccessControlPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.accessControl);
  }

  ledger(): TableHandle {
    return new TableHandle(this.page);
  }

  // The ledger's own search box. The first `.sp-toolbar__search` on the page
  // is the tree filter above it, which narrows the editor and not the table.
  async search(term: string): Promise<void> {
    await this.page.locator('#ac-filter-q').fill(term);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  async filterBy(name: string, value: string): Promise<void> {
    await this.page.locator(`form select[name="${name}"]`).first().selectOption(value);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The first real choice a toolbar select offers, after its "all" option. */
  async firstFilterValue(name: string): Promise<string> {
    const values = await this.page
      .locator(`form select[name="${name}"] option`)
      .evaluateAll((options) =>
        options.map((o) => (o as HTMLOptionElement).value).filter((v) => v !== ''),
      );
    if (values.length === 0) {
      throw new Error(`the ${name} filter offers no value to pick`);
    }
    return values[0];
  }

  // The sort indicator lives inside the `th`, so a sorted column's text is
  // "Person ▲" and an exact header match misses it. Matching on the label
  // prefix reads the column whether or not it is the one being sorted on.
  async columnValues(header: string, table = 0): Promise<string[]> {
    const root = this.page.locator('.sp-table').nth(table);
    const headers = await root.locator('.sp-table__el thead th').allInnerTexts();
    const index = headers.findIndex((t) => t.trim().toLowerCase().startsWith(header.toLowerCase()));
    if (index < 0) {
      throw new Error(`no column "${header}"; saw ${headers.join(' | ')}`);
    }
    return root.locator(`.sp-table__el tbody tr td:nth-child(${index + 1})`).allInnerTexts();
  }

  tree(): Locator {
    return this.page.locator('.sp-ac-tree');
  }

  groupRow(label: string): Locator {
    return this.page.locator('[data-action="select-dept"]').filter({ hasText: label }).first();
  }

  editorPane(): Locator {
    return this.page.locator('[data-pane="editor"]');
  }

  matrixPane(): Locator {
    return this.page.locator('[data-pane="matrix"]');
  }

  async selectGroup(label: string): Promise<void> {
    await this.groupRow(label).click();
  }
}
