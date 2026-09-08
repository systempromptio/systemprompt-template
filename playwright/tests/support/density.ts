// The measured bar: what a dense page actually looks like, in numbers.
//
// "Information-dense" is the whole point of this console and the easiest claim
// to lose quietly — a row grows two pixels a week and nobody notices until half
// the table is below the fold. These are the measurements that catch that, taken
// in the browser against the real rendered page rather than against the CSS.
//
// Every selector here is the design system's. If one is renamed, the metric it
// feeds silently reads zero and the bar passes — so they are named once, in
// SEL, and read from there.
import type { Page } from '@playwright/test';
import { SEL } from './pages/selectors';

export interface DensityMetrics {
  /** Rows of the PRIMARY table lying wholly inside the viewport. */
  rowsVisible: number;
  /** Rows of the primary table, so a thin dataset differs from a sparse layout. */
  rowsTotal: number;
  /** Height of the first row, the density figure itself. */
  rowH: number | null;
  /** Height of the KPI strip. */
  kpiH: number | null;
  /** Cells whose content visibly spills out of their own box. */
  overflowCells: number;
  /** Adjacent first-row cells whose rectangles intersect, within one table. */
  overlap: number;
  /** Rows whose first meaningful cell is ellipsized — a hidden name, not a short row. */
  truncatedPrimary: number;
  /** Header cells clipping their own label. */
  truncatedHeaders: number;
  /** Scroll containers wider than themselves. */
  hscroll: number;
  /** What is sticking out of each, so the cause is visible from the failure. */
  hscrollBy: string[];
  bodyScroll: boolean;
  tabs: number;
  dupTabs: boolean;
  /** Sub-labels reading a bare "0", which is a placeholder rather than a fact. */
  zeroSubs: number;
  /** Stacked filter rows. Two is a toolbar and a scope row; three is a wall. */
  filterRows: number;
  /** Which of the four counted, so a page author can see where the budget went. */
  filterRowsBy: Record<string, number>;
  crumbs: boolean;
  noHeader: boolean;
}

export async function densityMetrics(page: Page): Promise<DensityMetrics> {
  return page.evaluate((sel) => {
    const viewportHeight = window.innerHeight;
    const wholly = (el: Element) => {
      const b = el.getBoundingClientRect();
      return b.top >= 0 && b.bottom <= viewportHeight;
    };
    // Why the first table only: a page is entitled to a secondary listing, and
    // counting every table's rows together made "12 rows above the fold" a test
    // of how many tables a page has. The primary table is the one the page is
    // about, and the one the bar is about.
    const primaryTable = document.querySelector(sel.tableEl);
    const rows = primaryTable
      ? Array.from(primaryTable.querySelectorAll(':scope > tbody > tr'))
      : [];
    const cells = Array.from(document.querySelectorAll(`${sel.tableEl} td`));
    // Why grouped by table: a flat querySelectorAll across the page returns the
    // first row of every table concatenated, so the last cell of one table is
    // compared against the first cell of the next and always "overlaps" — a
    // page with two tables reported a defect it did not have, and a real
    // overlap inside the second table would have been invisible behind it.
    const tables = Array.from(document.querySelectorAll(sel.tableEl));
    const clipped = (el: Element) =>
      el.scrollWidth > el.clientWidth + 1 ||
      Array.from(el.querySelectorAll('*')).some((d) => d.scrollWidth > d.clientWidth + 1);

    let overlap = 0;
    let truncatedPrimary = 0;
    let truncatedHeaders = 0;
    for (const table of tables) {
      const firstRowCells = Array.from(
        table.querySelectorAll(':scope > tbody > tr:first-child > td'),
      ).filter((cell) => cell.getClientRects().length > 0);
      for (let i = 1; i < firstRowCells.length; i += 1) {
        const a = firstRowCells[i - 1].getBoundingClientRect();
        const b = firstRowCells[i].getBoundingClientRect();
        if (a.right > b.left + 2) overlap += 1;
      }
      for (const row of Array.from(table.querySelectorAll(':scope > tbody > tr'))) {
        const cells = Array.from(row.querySelectorAll(':scope > td'));
        // A leading checkbox is not the row's subject; the cell after it is.
        const primary = cells.find((c) => !c.querySelector('input[type=checkbox]'));
        if (primary && clipped(primary)) truncatedPrimary += 1;
      }
      for (const th of Array.from(table.querySelectorAll(':scope > thead th'))) {
        if (th.scrollWidth > th.clientWidth + 1) truncatedHeaders += 1;
      }
    }
    const overflowing = Array.from(
      document.querySelectorAll(`${sel.tableScroll}, .sp-shell__main`),
    ).filter((el) => el.scrollWidth > el.clientWidth + 2);
    const kpi = document.querySelector(sel.kpiGrid);
    const tabs = Array.from(document.querySelectorAll('[role=tab]')).map((t) =>
      t.textContent?.trim(),
    );
    return {
      rowsVisible: rows.filter(wholly).length,
      rowsTotal: rows.length,
      rowH: rows[0] ? Math.round(rows[0].getBoundingClientRect().height) : null,
      kpiH: kpi ? Math.round(kpi.getBoundingClientRect().height) : null,
      // A cell clamped with overflow:hidden and an ellipsis has scrollWidth
      // greater than clientWidth by design — that is the truncation working,
      // not a defect. Only a cell whose overflow is visible actually spills its
      // content across its neighbour, so the computed overflow decides.
      overflowCells: cells.filter((td) => {
        const bigger =
          td.scrollWidth > td.clientWidth + 2 || td.scrollHeight > td.clientHeight + 2;
        if (!bigger) return false;
        const style = getComputedStyle(td);
        return style.overflow === 'visible' || style.overflowX === 'visible';
      }).length,
      overlap,
      // Deliberately counted even though overflowCells ignores a clamped cell.
      // Truncation is a legitimate way to hold a row short — except on the value
      // that says which row this is, and on the header that says what the column
      // means. Hiding those two is how a fixed layout buys density the reader
      // pays for, and the bar must not reward it.
      truncatedPrimary,
      truncatedHeaders,
      hscroll: overflowing.length,
      // Why name the culprit: an element parked off-canvas still contributes its
      // width to an ancestor's scrollWidth, so a table that fits exactly can
      // report a sideways scroll caused by a closed drawer 640px to the right.
      // The number alone sends you to the table; the widest overhanging child
      // sends you to the drawer.
      hscrollBy: overflowing.map((el) => {
        const box = el.getBoundingClientRect();
        let worst = '';
        let overhang = 0;
        for (const child of Array.from(el.querySelectorAll('*'))) {
          const rect = child.getBoundingClientRect();
          const past = Math.round(rect.right - box.right);
          if (past > overhang) {
            overhang = past;
            const cls = child.className && typeof child.className === 'string'
              ? `.${child.className.trim().split(/\s+/).join('.')}`
              : '';
            worst = `${child.tagName.toLowerCase()}${cls}`;
          }
        }
        return overhang > 0 ? `${worst} overhangs by ${overhang}px` : 'no single child overhangs';
      }),
      bodyScroll: document.documentElement.scrollWidth > document.documentElement.clientWidth,
      tabs: tabs.length,
      dupTabs: new Set(tabs).size < tabs.length,
      zeroSubs: Array.from(
        document.querySelectorAll('.sp-kpi__note, .sp-kpi__delta, .sp-section__count'),
      ).filter((e) => e.textContent?.trim() === '0').length,
      filterRows: document.querySelectorAll(
        '.sp-toolbar, .sp-scope-filter, .sp-filter-ribbon, form.sp-filters',
      ).length,
      // Broken down because the shell's own scope bar spends one of the two
      // before the page draws anything, and a page author meeting the cap for
      // the first time has no way to guess that half their budget was already
      // gone.
      filterRowsBy: {
        '.sp-toolbar': document.querySelectorAll('.sp-toolbar').length,
        '.sp-scope-filter': document.querySelectorAll('.sp-scope-filter').length,
        '.sp-filter-ribbon': document.querySelectorAll('.sp-filter-ribbon').length,
        'form.sp-filters': document.querySelectorAll('form.sp-filters').length,
      },
      crumbs: !!document.querySelector(`${sel.breadcrumb}, nav[aria-label=Breadcrumb]`),
      noHeader: !document.querySelector(sel.pageHeader),
    };
  }, {
    tableRow: SEL.tableRow,
    tableEl: SEL.tableEl,
    tableScroll: SEL.tableScroll,
    kpiGrid: SEL.kpiGrid,
    breadcrumb: SEL.breadcrumb,
    pageHeader: SEL.pageHeader,
  });
}
