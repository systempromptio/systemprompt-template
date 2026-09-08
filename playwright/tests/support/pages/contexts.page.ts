// The contexts list and one context's detail page.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class ContextsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.contexts);
  }

  contextLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/contexts/"]`);
  }

  async openFirst(): Promise<ContextDetailPage> {
    await this.contextLinks().first().click();
    await expect(this.page).toHaveURL(/\/admin\/contexts\/.+/);
    return new ContextDetailPage(this.page, this.page.url());
  }

  /** Type into the page's own search box and submit.
   *
   *  Not `TableHandle.filter`: that reaches for `.sp-toolbar__search input`,
   *  and this page's search sits in the toolbar's filters slot beside the two
   *  selects it submits with.
   */
  async search(text: string): Promise<void> {
    const form = this.page.locator('#contexts-filter-form');
    await form.locator('input[name="q"]').fill(text);
    await form.getByRole('button', { name: /apply/i }).click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The "By user" / "Contexts" view switch. */
  viewTab(name: 'contexts' | 'users'): Locator {
    return this.page.locator(`${SEL.tab}[href*="view=${name}"]`);
  }

  /** Click a sortable column header by its label and wait for the re-render.
   *
   *  Not `TableHandle.sortBy`: the design system renders the sort indicator
   *  inside the `th`, so the header's text is "COST ↕" and the shared handle's
   *  exact-match column lookup never finds it.
   */
  async sortBy(label: string): Promise<string | null> {
    const th = this.page.locator(`${SEL.tableHeaderCell}`).filter({ hasText: new RegExp(`^\\s*${label}`, 'i') }).first();
    await th.getByRole('link').first().click();
    await this.page.waitForLoadState('networkidle');
    return th.getAttribute('aria-sort');
  }

}

export class ContextDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  /** What the session touched: files, tools and repositories, busiest first. */
  entityTable(): Locator {
    return this.page
      .locator(SEL.table)
      .filter({ has: this.page.locator('caption', { hasText: /Entities linked/i }) });
  }

  entityBars(): Locator {
    return this.page.locator('.sp-p-context-detail__bar');
  }
}
