// The request log and one request's audit detail.
//
// Only what is genuinely unique to these two pages lives here: the filter
// selects the log adds beside the shared search box and the detail page's two
// evidence sections. Rows, sorting, paging and the empty state come from
// TableHandle on the base object.
import { expect, type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class RequestsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.requests);
  }

  /** The log's own filter form — the one with the facet selects. */
  form(): Locator {
    return this.page.locator('form.sp-toolbar[role="search"]');
  }

  /** Choose one facet value and wait for the server to re-render. */
  async filterBy(field: 'model' | 'provider' | 'status' | 'tool', value: string) {
    await this.form().locator(`select[name="${field}"]`).selectOption({ value });
    await this.page.waitForLoadState('networkidle');
  }

  /** The active-filter chips above the table, by their visible value. */
  chips(): Locator {
    return this.page.locator('.sp-filter-ribbon__chip-value');
  }

  async clearFilters(): Promise<void> {
    await this.page.locator('.sp-filter-ribbon__clear').first().click();
    await this.page.waitForLoadState('networkidle');
  }

  rows(): Locator {
    return this.page.locator('tr[data-request-row]');
  }

  /** Open the audit detail for the first row by its time link. */
  async openFirstRow(): Promise<void> {
    await this.rows().first().locator('a').first().click();
    await expect(this.page.locator('h1').first()).toBeVisible();
  }
}

export class RequestDetailPage extends BasePage {
  constructor(page: Page, id: string) {
    super(page, PATHS.request(id));
  }

  section(label: string): Locator {
    return this.page.locator(`section[aria-label="${label}"]`);
  }

  safetyFindings(): Locator {
    return this.section('Safety findings');
  }

  toolCalls(): Locator {
    return this.section('Tool calls');
  }
}
