# Delivery Summary — Requirements Register Pass (2026-08-25)

Production is live at https://astound.systemprompt.io (commit `93f9a2de`, Fly app
sp-a2f658d8bc5f). This file is the wrap-up of the register delivery pass; the
per-row detail lives in [`REQUIREMENTS-DELIVERY-STATUS.md`](REQUIREMENTS-DELIVERY-STATUS.md)
and [`requirements/compliance-register.md`](requirements/compliance-register.md).

## What shipped

**Register.** All 43 rows re-assessed against HEAD with code evidence. Updated
stakeholder CSV at `requirements/register-updated-2026-08-25.csv` (statuses use
b.gulyaev's Testing/Delivered vocabulary, every row carries an "Ed's comments"
approach/timeline). Net position: ~23 rows Testing/Delivered, ~12 partial with
named gaps, 4 not delivered (019 semantic cache, 032 provider failover, 035 A/B
testing, 039 prompt versioning — scoped with estimates in
`requirements/design-finops-and-observability.md`, tracked as honest
`test.fixme` placeholders and on the public roadmap page).

**Built this pass.**
- FinOps: spend forecast + projected-overrun Slack alerts (REQ-011/014), weekly
  `cost_digest` job (016), web CSV export on both reports (015).
- Observability: configurable latency SLO with breach share (029), persisted
  hourly `usage_anomaly` job with alerts + dashboard surface (031).
- Access: 30-day expiring share tokens (025), `pii_extended` scanner
  (SSN/E.164) + transcript SSN redaction (036), Salesforce deprovisioning
  reconciliation job — IdP removal disables the account and revokes
  sessions/PATs (023).
- Enablement: governance chain (4 stages), safety scanners, and quota windows
  are now ON (`services/governance/config.yaml`, `services/gateway/policies.yaml`)
  — first-pass category set and quota sizes, review against real traffic.
- Organizations: `ai-kit` and `ai-sdlc-delivery` as isolated orgs on the
  standard plan (b.gulyaev's two-group split); onboarding is admin invites.
- Residency/no-retain routing (037/038) via core 0.39 `governance:` metadata +
  route `requires:` (sister session; patch-active, awaiting core release).
- Nine documentation pages at `/documentation/enterprise-*` with embedded
  evidence screenshots and replication commands.

**Verification.** Static/lint gates, unit (incl. 8 new tests), integration
676/676, contract 76/76 (baseline re-recorded: +2 CSV routes), e2e 94 passed +
4 fixme placeholders, fork-drift gate fully green (both repos synced
bidirectionally). Three latent defects found and fixed along the way: a
template-DB-pollution-dependent fixture, a UUID assumption in the ported
entity-access boundary, and an error-message regression in the rule handler.

## Open items

1. **Coverage ratchet is red — decision needed.** Total 79.42% vs the recorded
   81.17% baseline (80 floor). The gap is the new jobs' Slack/Salesforce
   delivery paths (`extensions/web/jobs` 64.55%), unreachable from unit tests.
   Either re-record `coverage/baseline.json` (review-visible act) or add
   harness-level job tests (~half a day).
2. **Core 0.39.0 release.** The `[patch.crates-io]` is ACTIVE against
   `../systemprompt-core` 0.39.0; publish core, then re-comment the patch.
   Pins are already synced (the 0.38 pins had silently disabled the patch —
   fixed in `93f9a2de`).
3. **macOS bridge installer** never published; deploy used
   `FETCH_ALLOW_MISSING=1`. Build with `just bridge-package-macos` when wanted.
4. **Decisions Astound owes** (block the remaining partial rows): geographic
   Hub dimension implementation (defined, next pass), authoritative SCM +
   identity mapping (008), safety category/quota sign-off, quality baseline
   (017/018), observability target stack (028).
