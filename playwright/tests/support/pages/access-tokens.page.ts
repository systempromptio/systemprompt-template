// The access-tokens page: every personal access token on the instance, whose
// it is, and whether it is still live.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class AccessTokensPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.accessTokens);
  }

  /** The state badges in the table, live/expired/revoked. */
  stateBadges(): Locator {
    return this.page.locator(`${SEL.tableRow} ${SEL.badge}`);
  }

  /** The row for one token, by the name it was issued under. */
  row(name: string): Locator {
    return this.page.locator(SEL.tableRow).filter({ hasText: name }).first();
  }

  /** The revoke control on a row, if the page offers one. */
  revokeButton(name: string): Locator {
    return this.row(name).getByRole('button', { name: /revoke/i });
  }
}
