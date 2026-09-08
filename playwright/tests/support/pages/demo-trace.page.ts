// The trace demo page: a scripted walk through one governed tool call, from
// the hook firing to the audit row it leaves behind.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';

export class DemoTracePage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.demoTrace);
  }

  /** The steps of the walk, in order. */
  steps(): Locator {
    return this.page.locator('ol li, .sp-table__el tbody tr');
  }

  /** The control that runs the demo call, if the page offers one. */
  runButton(): Locator {
    return this.page.getByRole('button', { name: /run|start|fire/i }).first();
  }
}
