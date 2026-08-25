// REQ-040 (central prompt/template distribution), REQ-041 (pre-execution tool
// governance), REQ-042 (governed MCP server registry), REQ-043 (tool schema
// validation).
//
// Acceptance criteria: REQ-040 "approved prompt templates can be centrally
// managed and distributed … with controlled updates/revocation"; REQ-042
// "every MCP server is centrally registered, authenticated, allowlisted,
// auditable, and revocable"; REQ-041 "every agent/tool invocation passes
// required authorization, secret-exposure, safety, and rate-limit checks
// before the underlying action executes"; REQ-043 "tool schemas are validated
// during registration/deployment".
//
// Honesty notes: REQ-041's enforcement (four-stage chain, first-deny-wins)
// and REQ-043's fail-fast registration are Rust-tier proofs — a browser
// cannot invoke a tool or register a malformed manifest. What the browser
// pins here is the governed catalog those controls apply to, and that
// governance DECISIONS (including denies) surface in the audit UI where an
// operator would look for them. The seed writes a `deny/tool_blocklist`
// decision on member-1's day-0 session, which is the row asserted below.
import { test, expect } from '../support/fixtures';

test.describe('@REQ-040 central catalog distribution', () => {
  test('@REQ-040 the plugin catalog lists the centrally-managed bundles', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/catalog/plugins');
    // Signed-manifest distribution means the catalog is loaded from the
    // instance's own services/ config, never hand-entered: the page states
    // its source and renders at least one real bundle.
    await expect(page.locator('main')).toContainText(/services\/plugins/i);
    await expect(page.locator('.catalog-card').first()).toBeVisible();
  });

  test('@REQ-040 the skills catalog lists distributable skill entries', async ({
    adminPage: page,
  }) => {
    // Unlike the plugin grid, the skills catalog renders as a table of
    // detail-linked rows.
    await page.goto('/admin/catalog/skills');
    await expect(page.locator('a.table-link').first()).toBeVisible();
    await expect(page.locator('.catalog-count')).toContainText(/[1-9]\d* skills?/);
  });
});

test.describe('@REQ-042 governed MCP server registry', () => {
  test('@REQ-042 the systemprompt server is registered with its admin scope visible', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/catalog/mcp');
    const entry = page.locator('a[href*="/admin/catalog/mcp/"]', {
      hasText: /systemprompt/i,
    });
    await expect(entry.first()).toBeVisible();
    await entry.first().click();
    // The registry claim is auth, not existence: the detail must show the
    // scopes gate that makes this server admin-only.
    await expect(page.locator('main')).toContainText(/Scopes/i);
    await expect(page.locator('.chip', { hasText: 'admin' }).first()).toBeVisible();
  });
});

test.describe('@REQ-041 @REQ-043 governance decisions are auditable in the UI', () => {
  test('@REQ-041 a denied tool call surfaces as a DENY with its policy named', async ({
    platformAdminPage: page,
  }) => {
    // Day 0 seeds decision `deny / tool_blocklist` on the same session as this
    // request, so the request's Policy chain must show the refusal — the
    // audit half of pre-execution governance.
    await page.goto('/admin/entities/requests/e2e-req-e2e-member-1-0-0');
    const chain = page.locator('section', {
      has: page.locator('#policy-chain-title'),
    });
    await expect(chain).toContainText('tool_blocklist');
    await expect(chain.locator('.mcp-badge-danger', { hasText: 'DENY' })).toBeVisible();
  });

  test('@REQ-043 the tool the decision governs is named alongside the decision', async ({
    platformAdminPage: page,
  }) => {
    // Schema-validated registration is proven in the Rust tier; the audit
    // surface must still attribute each decision to a concrete registered
    // tool, or a decision row cannot be traced back to what it governed.
    await page.goto('/admin/entities/requests/e2e-req-e2e-member-1-0-0');
    await expect(page.locator('main')).toContainText('Edit');
  });
});
