// REQ-018 (cost/quality/latency model optimization), REQ-030 (central content
// safety guardrails), REQ-033 (per-consumer rate limiting), REQ-036 (PII
// detection & redaction).
//
// These four are enforcement requirements whose proof lives in the Rust tier
// (tests/integration/gateway): driving a quota window to 429 (REQ-033),
// tripping the jailbreak/PII/secret scanners (REQ-030/036), and exercising
// request-shape routing (REQ-018) all require real gateway traffic a browser
// session cannot generate honestly. Nothing in this file pretends to prove
// enforcement. What it pins is the operator-visible half each requirement
// also demands: the telemetry that routing decisions consume, the audit
// surface where safety and rate-limit denials land, and the per-model data
// without which none of these policies could be reviewed.
//
// Register status honesty: REQ-018 is Partially Delivered (declarative
// routing + telemetry, no automatic optimizer), REQ-030/036 are Partially
// Delivered (heuristic coverage, block-not-redact in flight), REQ-033 is
// Delivered at user/org scope.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-018 routing inputs: cost and latency telemetry per model', () => {
  test('@REQ-018 per-model latency and spend are reported for the same period', async ({
    platformAdminPage: page,
  }) => {
    // No automatic optimizer exists (register gap); the delivered half is
    // that every routing decision's inputs — measured latency and cost per
    // model — are reported, not vendor claims: the per-model breakdown on the
    // requests page and the SLO split on the spend tab.
    await page.goto('/admin/entities/requests?tab=models&preset=30d');
    await expect(page.locator('main')).toContainText('claude-opus-5');

    await page.goto('/admin/analytics?tab=spend&org=e2e-corp&slo_ms=2000');
    await expect(page.locator('main')).toContainText(/Within SLO/i);
  });

  test('@REQ-018 the routing seam itself is declarative and admin-editable', async ({
    request,
  }) => {
    const res = await request.get('/api/public/admin/gateway', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const cfg = (await res.json()) as { routes: { model_pattern: string }[] };
    // A preferred-model decision is applied by editing these routes, not by
    // redeploying applications — the substrate the future optimizer targets.
    expect(cfg.routes.length).toBeGreaterThan(0);
  });
});

test.describe('@REQ-030 @REQ-036 safety decisions are centrally auditable', () => {
  test('@REQ-030 a policy denial lands in the same audit chain as the request', async ({
    platformAdminPage: page,
  }) => {
    // Central guardrails only count if every deny is reviewable in one place.
    // The seeded day-0 session carries a deny decision; the request detail
    // must render it in the policy chain with its verdict badge.
    await page.goto('/admin/entities/requests/e2e-req-e2e-member-1-0-0');
    const chain = page.locator('section', { has: page.locator('#policy-chain-title') });
    await expect(chain).toBeVisible();
    await expect(chain.locator('.mcp-badge-danger', { hasText: 'DENY' })).toBeVisible();
  });

  test('@REQ-036 the audit surface is admin-only, so audited prompts cannot leak sideways', async ({
    userPage: page,
  }) => {
    // Display-layer redaction is only meaningful if the surface holding the
    // raw material refuses non-admins outright: a member must never reach
    // another user's audited request content.
    await page.goto('/admin/entities/requests/e2e-req-e2e-member-1-0-0');
    expect(page.url()).toContain('/admin/profile');
  });
});

test.describe('@REQ-033 rate-limit decisions share the audit spine', () => {
  test('@REQ-033 every governed request lands a status row the quota guard reads', async ({
    platformAdminPage: page,
  }) => {
    // The quota buckets count these same rows; a request log that dropped
    // per-request status would blind the limiter and the operator alike. The
    // 429-at-exhaustion proof is tests/integration/gateway's.
    await page.goto('/admin/entities/requests?tab=log&preset=30d');
    const log = page.getByRole('region', { name: 'Request log' });
    const rows = log.locator('tbody tr');
    expect(await rows.count()).toBeGreaterThan(0);
    await expect(log.locator('tbody')).toContainText(/completed|failed/i);
  });
});
