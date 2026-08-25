// Roadmap placeholders for the register rows assessed "Not Delivered"
// (requirements/register-updated-2026-08-25.csv). Each is a fixme, never a
// pass: the suite must report these as outstanding until the capability
// ships, at which point the fixme is replaced by a real spec.
import { test } from '../support/fixtures';

test.describe('@REQ-019 @REQ-032 @REQ-035 @REQ-039 roadmap placeholders', () => {
  test.fixme('@REQ-019 tenant-isolated semantic caching — not yet delivered', () => {
    // No semantic cache exists in the gateway; provider prompt-caching
    // passthrough is not a tenant cache. Est. 4-6 wks after build-vs-integrate.
  });

  test.fixme('@REQ-032 automatic provider failover — not yet delivered', () => {
    // No failover exists in the gateway dispatch path; the previously cited
    // fallback belongs to the agent AI service, not /v1 routing.
  });

  test.fixme('@REQ-035 A/B model & provider testing — not yet delivered', () => {
    // Route resolution is strictly first-match; no percentage split or
    // experiment assignment mechanism exists.
  });

  test.fixme('@REQ-039 prompt versioning / rollback — not yet delivered', () => {
    // No prompt registry: system-prompt overrides have no version history, so
    // rollback today means redeploying the profile.
  });
});
