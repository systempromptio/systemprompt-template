// The models page: what the gateway will route to, with the price it charges.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class ModelsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.models);
  }

  /** The row for one model, by its id. */
  row(model: string): Locator {
    return this.page.locator(SEL.tableRow).filter({ hasText: model }).first();
  }

  /** The provider badges, one per row. */
  providerBadges(): Locator {
    return this.page.locator(`${SEL.tableRow} ${SEL.badge}`);
  }
}
