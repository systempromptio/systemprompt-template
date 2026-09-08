// Accessibility assertion shared by every page spec.
//
// Serious and critical are the two axe severities that describe a barrier
// rather than a preference: a control with no accessible name, a contrast
// failure, a form field with no label. Those are treated as defects. Minor and
// moderate findings are reported in the failure message when there is one, but
// do not fail a run on their own.
import AxeBuilder from '@axe-core/playwright';
import { expect, type Page } from '@playwright/test';
import { densityMetrics } from './density';

export async function expectAccessible(page: Page): Promise<void> {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze();
  const blocking = results.violations.filter(
    (v) => v.impact === 'serious' || v.impact === 'critical',
  );
  const summary = blocking
    .map((v) => `${v.id} (${v.impact}, ${v.nodes.length} nodes): ${v.help}`)
    .join('\n');
  expect(blocking, `axe found blocking violations:\n${summary}`).toHaveLength(0);
}

/** What "dense enough" means, per page shape. */
const BAR = {
  list: { rowsVisible: 12, rowHeight: 36 },
  detail: { rowsVisible: 8, rowHeight: 36 },
  // Identity plus metadata occupies two lines on the migrated activity lists.
  stackedList: { rowsVisible: 6, rowHeight: 64 },
  // Connection forms lead with controls; their secondary tables sit below them.
  form: { rowsVisible: 0, rowHeight: 36 },
  // Why: a page whose body is a chart, with the table as its index below it.
  waterfall: { rowsVisible: 0, rowHeight: 36 },
} as const;

export type PageShape = keyof typeof BAR;

/** Assert the measured density bar for a page.
 *
 *  The row-count check is bounded by the rows that exist. A page showing three
 *  approvals because three are pending is dense; failing it for that would
 *  measure the dataset rather than the layout, and would push people to seed
 *  more rows to turn a spec green — which is the opposite of what this guards.
 *  Everything else is unconditional, because a tall row, a clipped cell or a
 *  sideways scroll is a defect at any row count.
 */
export async function expectDensity(page: Page, shape: PageShape): Promise<void> {
  const m = await densityMetrics(page);
  const context = `density: ${JSON.stringify(m)}`;

  const wanted = Math.min(BAR[shape].rowsVisible, m.rowsTotal);
  expect(m.rowsVisible, `${wanted} rows should fit above the fold — ${context}`).toBeGreaterThanOrEqual(wanted);

  if (m.rowH !== null) {
    expect(m.rowH, `row height — ${context}`).toBeLessThanOrEqual(BAR[shape].rowHeight);
  }
  if (m.kpiH !== null) {
    expect(m.kpiH, `KPI strip height — ${context}`).toBeLessThanOrEqual(110);
  }
  expect(m.overflowCells, `cells clipping their own content — ${context}`).toBe(0);
  expect(m.overlap, `overlapping cells in the first row — ${context}`).toBe(0);
  expect(
    m.truncatedPrimary,
    `rows whose name cell is ellipsized. Truncation is a fair way to keep a row ` +
      `short, but not on the value that says which row this is — widen that column ` +
      `and clip a less load-bearing one. ${context}`,
  ).toBe(0);
  expect(
    m.truncatedHeaders,
    `column headers clipping their own label — ${context}`,
  ).toBe(0);
  expect(
    m.hscroll,
    `containers scrolling sideways: ${m.hscrollBy.join('; ') || 'cause not isolated'}. ` +
      `An element parked off-canvas still adds its width to an ancestor's scroll, so ` +
      `a closed drawer can report this on a table that fits. ${context}`,
  ).toBe(0);
  expect(m.bodyScroll, `the page itself scrolls sideways — ${context}`).toBe(false);
  const filterBreakdown = Object.entries(m.filterRowsBy)
    .filter(([, n]) => n > 0)
    .map(([sel, n]) => `${sel} x${n}`)
    .join(', ');
  expect(
    m.filterRows,
    `too many stacked filter rows (${filterBreakdown}). The shell's scope bar ` +
      `spends one of the two before your page draws anything, so a page has one row ` +
      `for its own controls — fold the extras into the shared toolbar. ${context}`,
  ).toBeLessThanOrEqual(2);
  expect(m.dupTabs, `duplicate tab labels — ${context}`).toBe(false);
  expect(m.zeroSubs, `sub-labels reading a bare zero — ${context}`).toBe(0);
  expect(m.crumbs, `breadcrumbs are present — ${context}`).toBe(true);
  expect(m.noHeader, `the page has a design-system header — ${context}`).toBe(false);
}
