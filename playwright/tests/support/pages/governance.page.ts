// The governance policies page and one policy's editor.
//
// The policies are the four the chain runs, read from the live engine rather
// than the database, so the list is fixed-size and the only mutation is the
// enabled toggle on the editor — a POST that the spec drives and reverts.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class GovernancePage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.governance);
  }

  /** Every policy link in the table, in chain order. */
  policyLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/governance/policies/"]`);
  }

  row(policyId: string): Locator {
    return this.page.locator(SEL.tableRow).filter({ hasText: policyId }).first();
  }

  async open(policyId: string): Promise<GovernancePolicyPage> {
    await this.page.goto(PATHS.governancePolicy(policyId));
    await expect(this.page.locator('h1').first()).toBeVisible();
    return new GovernancePolicyPage(this.page, policyId);
  }
}

export class GovernancePolicyPage extends BasePage {
  constructor(page: Page, readonly policyId: string) {
    super(page, PATHS.governancePolicy(policyId));
  }

  /** The enable/disable form, which posts to the toggle route. */
  toggleForm(): Locator {
    return this.page.locator(`form[action="${PATHS.governancePolicy(this.policyId)}/toggle"]`);
  }
}
