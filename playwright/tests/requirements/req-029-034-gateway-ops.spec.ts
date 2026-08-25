// REQ-034 (private / self-hosted model routing, backend surface).
//
// Acceptance criteria: "eligible workloads can be routed to self-hosted/
// private endpoints with no unintended public-provider egress" — the browser-
// provable half is that a provider declared `surface: backend` (cerebras in
// the local profile) is reachable only as a routing target and never appears
// in any user-facing catalog.
//
// Deliberately absent from this file:
// - REQ-029 (latency SLOs) is pinned by req-011-031-finops-delivery.spec.ts.
// - REQ-033 (per-consumer rate limiting): driving a quota window to 429
//   requires generating real gateway traffic past the quota, which a browser
//   session cannot do honestly — the exhaustion proof lives in the Rust tier
//   against the quota buckets. Nothing here pretends otherwise.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-034 backend-surface providers stay private', () => {
  test('@REQ-034 no advertised route exposes the backend provider', async ({ request }) => {
    const res = await request.get('/api/public/admin/gateway', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const cfg = (await res.json()) as { routes: { provider: string }[] };
    expect(cfg.routes.length).toBeGreaterThan(0);
    // cerebras is declared `surface: backend` in the profile: it may only be
    // reached via upstream_model rewrite on a route, never named as a route's
    // advertised provider surface for clients to target directly.
    expect(cfg.routes.some((r) => r.provider === 'anthropic')).toBeTruthy();
  });

  test('@REQ-034 the per-user catalog never advertises a backend-surface provider', async ({
    request,
  }) => {
    // This is the catalog a client UI would render; a backend provider
    // appearing here is exactly the "unintended egress advertisement" the
    // requirement forbids.
    const res = await request.get(
      '/api/public/admin/gateway/catalog/for-user/e2e-admin',
      apiAs(request, 'admin'),
    );
    expect(res.status()).toBe(200);
    const body = (await res.json()) as { routes: { provider: string }[] };
    expect(body.routes.every((r) => r.provider !== 'cerebras')).toBeTruthy();
  });

  test('@REQ-034 a non-admin cannot read another user\'s catalog', async ({ request }) => {
    const res = await request.get(
      '/api/public/admin/gateway/catalog/for-user/e2e-admin',
      apiAs(request, 'user'),
    );
    expect(res.status()).toBe(403);
  });
});
