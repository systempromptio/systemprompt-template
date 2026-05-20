# CLI friction — template-side fixes

Companion to `/var/www/html/systemprompt-core/CLI_FRICTION_PLAN.md`. Together they close every item from the post-refactor CLI smoke test.

## Items

### T1. Enterprise-demo plugin references missing scripts
**Severity:** low (warning noise, not broken behavior)
**File:** `services/plugins/enterprise-demo/config.yaml:50-52`

`core plugins validate` reports:
```
Script 'seed-demo' not found at services/plugins/enterprise-demo/demo/01-seed-data.sh
Script 'sweep' not found at services/plugins/enterprise-demo/demo/sweep.sh
```

The `demo/` directory under that plugin doesn't exist — only `config.yaml` does. The references are dead.

**Fix:** Delete the two script entries from `config.yaml`. Don't restore the files — they're legacy from an earlier demo flow that's since been replaced by the top-level `demo/` scripts.

If the plugin has no other script entries after removal, drop the entire `scripts:` block (don't leave an empty list).

### T2. `web` extension migration tracking drift (`15/11 applied`)
**Severity:** low (confusing telemetry, no functional impact)
**Files:**
- `extensions/web/src/schemas.rs:25-27` — `extension_migrations!()` macro that declares the count
- `extensions/web/schema/migrations/` — on-disk `.sql` files (11)
- Migration tracking table rows for `extension='web'` (15)

The tracking table has 4 more applied rows than the extension currently declares. These are legacy migrations that were applied to dev databases but have since been consolidated/removed from the source tree.

**Fix:** Identify the 4 surplus tracking rows:
```bash
just cli infra db query "SELECT version, name FROM _schema_migrations WHERE extension='web' ORDER BY version"
```
For each row whose `version` no longer maps to a `.sql` file under `extensions/web/schema/migrations/`:
- If the migration's effect is now subsumed by a renamed/consolidated newer migration → use `systemprompt infra db migrate-mark-applied` semantics to retire the row cleanly (verify the CLI supports row deletion; if not, this becomes a core C12-class follow-up).
- If the migration was a no-op left over from cleanup → delete the tracking row via `infra db execute` with explicit operator confirmation.

Document the cleanup operation in `CHANGELOG.md` so other clones with the same drift know how to reconcile.

### T3. Stale text sweep in `services/` and `AGENTS.md`
**Severity:** low (drift)

While in the area, grep for stale references to features removed in the recent refactor wave:
- `"system" user`, `system@local`, `UserId::anonymous()` literal references — every mention should be either accurate or deleted.
- The `services/content/documentation/authentication.md` section from the prior W5 work should reference the new admin-role-based resolver (core item C1) once that lands, not the literal-`name='admin'` model.
- `AGENTS.md` line about identity should match whatever core's resolver actually does after C1.

Don't speculatively rewrite — only change strings that contradict the code as it ships after this plan.

## Verification

```bash
# T1
just cli core plugins validate
# Expect: enterprise-demo reports zero script errors.

# T2
just cli infra db migrate-status
# Expect: web shows N/N applied (equal numbers), no >100% drift.

# T3
rg -n '"system" user|system@local|UserId::anonymous' --type rust --type yaml --type md
# Expect: every hit is either gone or matches current behavior.
```

## Files touched

- `services/plugins/enterprise-demo/config.yaml`
- `_schema_migrations` table (data, via CLI)
- `CHANGELOG.md`
- Possibly `AGENTS.md`, `services/content/documentation/authentication.md` (depends on core C1 landing)

## Order

T1 (trivial) → T3 (grep sweep) → T2 (data cleanup, gated on operator confirmation).

## Out of scope

- Adding new CLI commands.
- Touching destructive job runners or services (start/stop/restart) — read-only smoke test focus.
- Core-side fixes — see `/var/www/html/systemprompt-core/CLI_FRICTION_PLAN.md`.
