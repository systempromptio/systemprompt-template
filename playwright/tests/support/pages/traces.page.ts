// The trace list and one trace's waterfall.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class TracesPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.traces);
  }

  traceLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/traces/"]`);
  }

  async openFirst(): Promise<TraceDetailPage> {
    await this.traceLinks().first().click();
    await expect(this.page).toHaveURL(/\/admin\/traces\/.+/);
    return new TraceDetailPage(this.page, this.page.url());
  }

  /** Narrow to one person through the toolbar's user facet and wait for the re-render. */
  async filterByUser(userId: string): Promise<void> {
    await this.page.locator('select[name="user_id"]').selectOption({ value: userId });
    await this.page.getByRole('button', { name: /apply/i }).click();
    await this.page.waitForLoadState('networkidle');
  }

  /** Row height in pixels, which is the density claim this page had to fix. */
  async rowHeight(): Promise<number> {
    const box = await this.table().rows().first().boundingBox();
    if (!box) throw new Error('no rows to measure');
    return box.height;
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

export class TraceDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  /** The waterfall is one inline SVG, not a stack of positioned divs. */
  chart(): Locator {
    return this.page.locator('svg.sp-p-trace-detail__svg');
  }

  bars(): Locator {
    return this.page.locator('rect.sp-p-trace-detail__bar');
  }

  spanTable(): Locator {
    return this.page
      .locator(SEL.table)
      .filter({ has: this.page.locator('caption', { hasText: /Spans in this trace/i }) });
  }

  /** Badges in the span table's status column, including REJECTED. */
  statusBadges(): Locator {
    return this.spanTable().locator(`tbody ${SEL.badge}`);
  }
}
