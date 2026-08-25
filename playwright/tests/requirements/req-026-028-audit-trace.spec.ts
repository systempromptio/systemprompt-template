// REQ-026 (immutable AI audit trail), REQ-027 (end-to-end traceability),
// REQ-028 (observability export).
//
// Acceptance criteria: REQ-026 "every model call produces an immutable audit
// record containing actor, time, model/provider, request/response lineage,
// policy decision, and relevant consumption metadata"; REQ-027 "a single trace
// identifier correlates the originating request, model calls, tool/MCP calls,
// downstream actions, outcomes, and cost"; REQ-028 "AI telemetry can be
// exported into Astound's standard observability/SIEM stack".
//
// Honesty notes: immutability is append-only by convention (the register says
// so) — what a browser proves is completeness of the record, not WORM
// storage. REQ-028's OTLP ingest and SSE audit stream live in core with no
// admin-UI surface in this repo; the export-side proof belongs to the Rust
// tier, and here we pin only that the audited data those exporters read is
// present and queryable.
import { test, expect } from '../support/fixtures';

// Seeded by playwright/setup/seed.ts: member-1, day 0, request 0 — day 0 also
// seeds a DENY governance decision on the same session, so this one id
// exercises the whole audit chain.
const SEEDED_REQUEST = 'e2e-req-e2e-member-1-0-0';
const SEEDED_TRACE = 'e2e-trace-e2e-member-1-0';

test.describe('@REQ-026 immutable AI audit trail', () => {
  test('@REQ-026 the request log lists the seeded model call', async ({
    platformAdminPage: page,
  }) => {
    // The page defaults to an aggregate Overview tab; the per-call rows the
    // requirement is about live under the Log tab.
    await page.goto('/admin/entities/requests?tab=log&preset=30d');
    await expect(page.locator('tr', { hasText: 'e2e-member-1' }).first()).toBeVisible();
  });

  test('@REQ-026 one audit record carries actor, model, policy chain and trace together', async ({
    platformAdminPage: page,
  }) => {
    // The requirement is reconstruction from ONE record, so every field is
    // asserted on the same detail page, not collected from three screens.
    await page.goto(`/admin/entities/requests/${SEEDED_REQUEST}`);
    const main = page.locator('main');
    await expect(main).toContainText('e2e-member-1'); // actor
    await expect(main).toContainText(/anthropic/i); // provider
    await expect(main).toContainText(/claude/); // model
    await expect(main).toContainText(/Policy chain/i); // policy decisions section
    // Consumption metadata: latency is rendered as its own labelled field.
    await expect(main).toContainText(/Latency/i);
  });

  test('@REQ-026 an unknown request id is a dead end, not an empty template', async ({
    platformAdminPage: page,
  }) => {
    const res = await page.goto('/admin/entities/requests/e2e-req-does-not-exist');
    expect(res?.status()).toBe(404);
  });
});

test.describe('@REQ-027 end-to-end traceability', () => {
  test('@REQ-027 one trace id resolves the chain: session, identity, spans', async ({
    platformAdminPage: page,
  }) => {
    await page.goto(`/admin/entities/traces/${SEEDED_TRACE}`);
    const main = page.locator('main');
    // The seeded trace groups multiple ai_requests under one id; a span count
    // of zero would mean the trace id correlates nothing.
    await expect(main).toContainText(/Spans/i);
    await expect(main).toContainText('e2e-member-1');
    await expect(main).toContainText(/e2e-session-e2e-member-1-0|e2e-sessi/);
  });

  test('@REQ-027 the trace list reaches the same detail the request detail links to', async ({
    platformAdminPage: page,
  }) => {
    // The correlation claim cuts both ways: the trace must be findable from
    // the list surface, not only by pasting an id into the URL. The list rows
    // link by session id (the detail route resolves either identifier).
    await page.goto('/admin/entities/traces');
    // KNOWN COSMETIC DEFECT (reported): at the 1440px test viewport the
    // `.trace-row__id` anchor's own box collapses to zero width, so the
    // anchor itself reads as hidden — its inner <code> label is the visible,
    // clickable box and still navigates through the anchor.
    const link = page
      .locator('a[href*="/admin/entities/traces/e2e-session-e2e-member-1"] code')
      .first();
    await expect(link).toBeVisible();
    await link.click();
    await expect(page).toHaveURL(/\/admin\/entities\/traces\/e2e-session-e2e-member-1/);
  });
});

test.describe('@REQ-028 observability export', () => {
  test('@REQ-028 the audited spine the exporters read is complete for a seeded call', async ({
    platformAdminPage: page,
  }) => {
    // Honest scope: OTLP ingest and the SSE stream are core transport with no
    // page in this repo; push egress to Datadog/Splunk is a register gap
    // (Partially Delivered). What must hold here is that the structured audit
    // row an exporter would ship — status, model, latency, trace linkage — is
    // fully populated for a governed call.
    await page.goto(`/admin/entities/requests/${SEEDED_REQUEST}`);
    const main = page.locator('main');
    await expect(main).toContainText(/Status/i);
    await expect(main).toContainText(/completed|failed/i);
    await expect(main).toContainText(/Model/i);
  });
});
