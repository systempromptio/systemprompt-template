// A handle over one rendered data table: rows, sorting, filtering, paging and
// CSV export. Every admin listing is the same component, so every listing spec
// drives it through this one object rather than re-deriving row selectors.
//
// `.sp-table` is the card and `.sp-table__el` the table inside it, so every
// row and header locator names the element — starting at the card alone would
// also match a nested table's rows.
import { expect, type Download, type Locator, type Page } from '@playwright/test';
import { SEL } from './selectors';

export class TableHandle {
  readonly locator: Locator;

  constructor(
    private readonly page: Page,
    readonly name?: string,
  ) {
    this.locator = name
      ? page.locator(`${SEL.table}[data-table="${name}"]`)
      : page.locator(SEL.table).first();
  }

  /** Data rows. The design system's empty row is a row too, and is excluded. */
  rows(): Locator {
    return this.locator.locator(SEL.tableRow).filter({ hasNot: this.page.locator(SEL.empty) });
  }

  async rowCount(): Promise<number> {
    if (await this.locator.locator(SEL.tableEmptyRow).count()) return 0;
    return this.rows().count();
  }

  headers(): Locator {
    return this.locator.locator(SEL.tableHeaderCell);
  }

  /** Row values for one column, addressed by its visible header text. */
  async column(header: string): Promise<string[]> {
    const index = await this.columnIndex(header);
    return this.rows().locator(`td:nth-child(${index + 1})`).allInnerTexts();
  }

  // A sort header's innerText carries the glyph and a newline alongside the
  // label, so an exact compare fails on every sortable column — reported as
  // 'no column "Person" ... saw PERSON', which reads as a case bug and is not
  // one. Compare the first line, case-folded, with runs of space collapsed.
  private static normalise(text: string): string {
    return text.split('\n')[0].replace(/\s+/g, ' ').trim().toLowerCase();
  }

  private async columnIndex(header: string): Promise<number> {
    const texts = await this.headers().allInnerTexts();
    const wanted = TableHandle.normalise(header);
    const index = texts.findIndex((t) => TableHandle.normalise(t) === wanted);
    if (index < 0) {
      throw new Error(
        `no column "${header}" in table ${this.name ?? '(first)'}; saw ${texts.join(' | ')}`,
      );
    }
    return index;
  }

  /** Click a column header to sort. The header carries the resulting aria-sort. */
  async sortBy(header: string): Promise<'ascending' | 'descending'> {
    const index = await this.columnIndex(header);
    const th = this.headers().nth(index);
    await th.click();
    await this.page.waitForLoadState('networkidle');
    const direction = await th.getAttribute('aria-sort');
    if (direction !== 'ascending' && direction !== 'descending') {
      throw new Error(`sorting by "${header}" left aria-sort as ${direction ?? 'absent'}`);
    }
    return direction;
  }

  /** The header currently sorted, if any, as its text and direction. */
  async sortedBy(): Promise<{ header: string; direction: string } | null> {
    const th = this.locator.locator(`${SEL.tableEl} ${SEL.tableSorted}`).first();
    if ((await th.count()) === 0) return null;
    return {
      header: (await th.innerText()).trim(),
      direction: (await th.getAttribute('aria-sort')) ?? '',
    };
  }

  async filter(text: string): Promise<void> {
    const input = this.page.locator(SEL.toolbarSearchInput).first();
    await input.fill(text);
    await input.press('Enter');
    await this.page.waitForLoadState('networkidle');
  }

  private controls(): Locator {
    return this.page.locator(SEL.paginationControls).first();
  }

  /** Follow the next or previous page control. A disabled one is not a link. */
  async turnPage(direction: 'next' | 'previous'): Promise<void> {
    const control = this.controls().getByRole('link', { name: new RegExp(direction, 'i') });
    await expect(control, `the ${direction} page control is not available`).toBeVisible();
    await control.click();
    await this.page.waitForLoadState('networkidle');
  }

  async nextPage(): Promise<void> {
    await this.turnPage('next');
  }

  async setPageSize(size: string): Promise<void> {
    await this.page.locator(SEL.paginationSize).first().selectOption(size);
    await this.page.waitForLoadState('networkidle');
  }

  async currentPage(): Promise<string> {
    return (await this.page.locator(SEL.paginationCurrent).first().innerText()).trim();
  }

  /** Trigger the CSV export and return the downloaded file's text. */
  async csv(): Promise<string> {
    // Some pages fill the toolbar's actions slot and some put their controls
    // straight in the toolbar, so this scopes to the toolbar itself.
    const link = this.page.locator(SEL.toolbar).getByRole('link', { name: /csv/i }).first();
    const [download] = await Promise.all([this.page.waitForEvent('download'), link.click()]);
    return readDownload(download);
  }

  async expectEmpty(): Promise<void> {
    await expect(this.page.locator(SEL.empty)).toBeVisible();
    expect(await this.rowCount()).toBe(0);
  }
}

async function readDownload(download: Download): Promise<string> {
  const stream = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks).toString('utf8');
}
