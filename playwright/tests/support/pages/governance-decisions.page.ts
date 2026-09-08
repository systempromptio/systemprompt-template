// The governance decision log: one row per chain decision over one window,
// narrowed by the toolbar's GET form.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class GovernanceDecisionsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.governanceDecisions);
  }

  /** Narrow the log through the toolbar's GET form. */
  async filterBy(name: string, value: string): Promise<void> {
    await this.page.locator(`form select[name="${name}"]`).first().selectOption(value);
    await this.page.waitForLoadState('networkidle');
  }

  /** The decision badges (ALLOW / WARN / DENY), one per row. */
  decisionBadges(): Locator {
    return this.page.locator(`${SEL.tableRow} ${SEL.badge}`);
  }

  reasonCells(): Locator {
    return this.page.locator('.sp-p-governance__reason');
  }
}
