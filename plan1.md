
● The plan covers 6 waves of 5 parallel agents each, decomposing the 416-file web extension into 5 sub-crates:
                                                                                          
  1. Wave 1 — Extract shared (models, error, config, utils) — foundation everything depends on                                                                                            
  2. Wave 2 — Extract admin (332 files) — biggest win, already architecturally isolated                                                                                                   
  3. Wave 3 — Extract content (repository, services, api) — clean layered deps                                                                                                            
  4. Wave 4 — Extract site (blog, docs, homepage, features, navigation, partials, assets) — presentation layer                                                                            
  5. Wave 5 — Extract jobs (18 files) — last because it cross-cuts admin and content                                                                                                      
  6. Wave 6 — Final verification, cleanup, clippy, tests                                                                                                                                  
                                                                                                                                                                                          
  Key decisions: root crate stays as the single Extension implementor; SecretCryptoError stays in admin (error.rs uses String instead of #[from]); include_str! paths use                 
  CARGO_MANIFEST_DIR for robustness.                                                                                                                                                      
                                                                                                                                                                                          
──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 Ready to code?

 Here is Claude's plan:
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 Stage 6: Web Extension Modular Decomposition

 Context

 The systemprompt-web-extension crate is 416 files / 57K LOC in a single crate. After Stages 1-5 cleaned up pool access, deduplication, transactions, and clippy compliance, this crate
 is ready to split into domain-focused sub-crates. The admin module alone is 332 files (83%) and is already architecturally isolated.

 Target Structure

 extensions/web/
 ├── shared/     → systemprompt-web-shared     (foundation: models, error, config, utils)
 ├── admin/      → systemprompt-web-admin      (332 files: handlers, repos, types, gamification, etc.)
 ├── content/    → systemprompt-web-content     (22 files: repository, services, api)
 ├── site/       → systemprompt-web-site        (25 files: blog, docs, homepage, features, navigation, partials, assets, extenders, schemas)
 ├── jobs/       → systemprompt-web-jobs        (18 files: all background jobs)
 └── src/        → systemprompt-web-extension   (thin orchestrator: extension.rs, extension_impl.rs, lib.rs)

 Architecture Decision

 Single Extension implementor: The root crate remains the sole Extension trait implementor. Sub-crates are pure libraries exporting types, functions, and router constructors. Root
 re-exports sub-crate modules via pub use so existing external consumers see no change.

 Dependency DAG

 shared (no local deps)
   ├── content (shared)
   ├── site (shared)
   ├── admin (shared)
   └── jobs (shared + content + admin)
         └── root (all 5 sub-crates, implements Extension)

 Key Design Decisions

 1. SecretCryptoError stays in admin — It depends on chacha20poly1305 and systemprompt::models::SecretsBootstrap. In error.rs, change Crypto(#[from] SecretCryptoError) to Crypto(String)
  and convert at call sites with .map_err(|e| MarketplaceError::Crypto(e.to_string())).
 2. config_loader.rs stays in root — It imports from features, homepage, navigation (site crate) and is consumed by extension.rs (root). Root depends on site, so this works.
 3. schemas.rs stays in root — It uses include_str!("../schema/*.sql") relative to extensions/web/src/. The schema/ directory is at extensions/web/schema/. Keeping schemas in root
 avoids path breakage.
 4. include_str! paths for partials/assets — When moved to site/src/, paths like ../../../services/web/templates/ become ../../../../services/web/templates/ (one level deeper).
 Alternative: use concat!(env!("CARGO_MANIFEST_DIR"), "/../../services/web/templates/...").

 ---
 WAVE 1: Extract shared crate (Foundation)

 All 5 agents work on non-overlapping file sets. Must complete before any other wave.

 ┌───────┬────────────────────────────────────────────────────────────────────────────────────────┬────────────────────────────────────────────────────────────┐
 │ Agent │                                          Task                                          │                           Files                            │
 ├───────┼────────────────────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────┤
 │ 1-A   │ Create shared/ crate scaffold + move models/ (9 files)                                 │ shared/Cargo.toml, shared/src/lib.rs, shared/src/models/** │
 ├───────┼────────────────────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────┤
 │ 1-B   │ Move error.rs + utils.rs to shared; refactor SecretCryptoError out of MarketplaceError │ shared/src/error.rs, shared/src/utils.rs                   │
 ├───────┼────────────────────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────┤
 │ 1-C   │ Move config.rs + config_errors.rs to shared                                            │ shared/src/config.rs, shared/src/config_errors.rs          │
 ├───────┼────────────────────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────┤
 │ 1-D   │ Update workspace Cargo.toml + root Cargo.toml                                          │ Cargo.toml, extensions/web/Cargo.toml                      │
 ├───────┼────────────────────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────┤
 │ 1-E   │ Update root lib.rs — remove moved modules, add re-exports from shared                  │ extensions/web/src/lib.rs                                  │
 └───────┴────────────────────────────────────────────────────────────────────────────────────────┴────────────────────────────────────────────────────────────┘

 Shared Cargo.toml deps: systemprompt, serde, serde_json, serde_yaml, chrono, uuid, sqlx, url, thiserror, axum, tracing

 Import changes: All use crate::models:: → use systemprompt_web_shared::models:: (but root re-exports make crate::models still work within root crate)

 Verify: cargo check -p systemprompt-web-shared && cargo check -p systemprompt-web-extension

 ---
 WAVE 2: Extract admin crate (332 files — biggest win)

 Admin is isolated (only imports error.rs from shared). Move the entire admin/ directory tree.

 ┌───────┬──────────────────────────────────────────────────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────────────────┐
 │ Agent │                               Task                               │                                                 Files                                                  │
 ├───────┼──────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2-A   │ Create admin/ crate scaffold + move top-level admin files        │ admin/Cargo.toml, admin/src/lib.rs, mod.rs→lib.rs, middleware.rs, numeric.rs, event_hub/,              │
 │       │                                                                  │ slack_alerts/, templates/, tier_enforcement/, tier_limits/                                             │
 ├───────┼──────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2-B   │ Move admin/handlers/ (all API + SSR handlers, ~80 files)         │ admin/src/handlers/**                                                                                  │
 ├───────┼──────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2-C   │ Move admin/repositories/ (~63 files)                             │ admin/src/repositories/**                                                                              │
 ├───────┼──────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2-D   │ Move admin/types/ + admin/activity/ + admin/gamification/ +      │ admin/src/types/**, admin/src/activity/**, admin/src/gamification/**, admin/src/routes/**              │
 │       │ admin/routes/ (~40 files)                                        │                                                                                                        │
 ├───────┼──────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2-E   │ Update workspace + root crate; wire pub use                      │ Cargo.toml, extensions/web/Cargo.toml, extensions/web/src/lib.rs                                       │
 │       │ systemprompt_web_admin as admin; in root lib.rs                  │                                                                                                        │
 └───────┴──────────────────────────────────────────────────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────────────────┘

 Import rewrites inside admin crate:
 - use crate::admin::X → use crate::X (now top-level within admin crate)
 - use crate::error:: → use systemprompt_web_shared::error::
 - use crate::models:: → use systemprompt_web_shared::models:: (if any)

 Admin Cargo.toml deps: systemprompt-web-shared, systemprompt, axum, sqlx, tokio, serde, serde_json, serde_yaml, async-trait, tracing, chrono, uuid, thiserror, sha2, chacha20poly1305,
 hex, base64, reqwest, rand, handlebars, indexmap, chrono-tz, url, urlencoding, flate2, tar, zip, tempfile, walkdir, comrak, futures, tower, tower-http, async-stream, tokio-stream,
 anyhow

 Verify: cargo check -p systemprompt-web-admin && cargo check -p systemprompt-web-extension

 ---
 WAVE 3: Extract content crate (22 files)

 Content layer: repository → services → api. Clean layered deps.

 ┌───────┬─────────────────────────────────────────────────────────────────────────┬───────────────────────────────────────────────────────────────────┐
 │ Agent │                                  Task                                   │                               Files                               │
 ├───────┼─────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────┤
 │ 3-A   │ Create content/ crate scaffold + move repository/ (7 files)             │ content/Cargo.toml, content/src/lib.rs, content/src/repository/** │
 ├───────┼─────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────┤
 │ 3-B   │ Move services/ (8 files)                                                │ content/src/services/**                                           │
 ├───────┼─────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────┤
 │ 3-C   │ Move api/ (7 files)                                                     │ content/src/api/**                                                │
 ├───────┼─────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────┤
 │ 3-D   │ Update workspace + root Cargo.toml                                      │ Cargo.toml, extensions/web/Cargo.toml                             │
 ├───────┼─────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────┤
 │ 3-E   │ Update root lib.rs re-exports; verify jobs still compile via re-exports │ extensions/web/src/lib.rs                                         │
 └───────┴─────────────────────────────────────────────────────────────────────────┴───────────────────────────────────────────────────────────────────┘

 Import rewrites:
 - use crate::models:: → use systemprompt_web_shared::models::
 - use crate::error:: → use systemprompt_web_shared::error::
 - use crate::config:: → use systemprompt_web_shared::config::
 - use crate::repository:: → use crate::repository:: (internal to content crate)
 - use crate::services:: → use crate::services:: (internal to content crate)

 Content Cargo.toml deps: systemprompt-web-shared, systemprompt, sqlx, axum, serde, serde_json, chrono, uuid, tracing, async-trait, reqwest, pulldown-cmark, comrak, sha2, url,
 urlencoding, walkdir, anyhow

 Verify: cargo check -p systemprompt-web-content && cargo check -p systemprompt-web-extension

 ---
 WAVE 4: Extract site crate (25 files)

 Site rendering: blog, docs, homepage, features, navigation + partials, assets, extenders.

 ┌───────┬───────────────────────────────────────────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────────┐
 │ Agent │                                     Task                                      │                                   Files                                   │
 ├───────┼───────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
 │ 4-A   │ Create site/ crate scaffold + move blog/ (5 files) + docs/ (5 files)          │ site/Cargo.toml, site/src/lib.rs, site/src/blog/**, site/src/docs/**      │
 ├───────┼───────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
 │ 4-B   │ Move homepage/ (5 files) + features/ (3 files)                                │ site/src/homepage/**, site/src/features/**                                │
 ├───────┼───────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
 │ 4-C   │ Move navigation/ (3 files) + extenders/ (2 files)                             │ site/src/navigation/**, site/src/extenders/**                             │
 ├───────┼───────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
 │ 4-D   │ Move partials.rs + partials_animations.rs + assets.rs; fix include_str! paths │ site/src/partials.rs, site/src/partials_animations.rs, site/src/assets.rs │
 ├───────┼───────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
 │ 4-E   │ Update workspace + root crate; wire re-exports                                │ Cargo.toml, extensions/web/Cargo.toml, extensions/web/src/lib.rs          │
 └───────┴───────────────────────────────────────────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────────┘

 include_str! path fixes (Agent 4-D):
 - Current: include_str!("../../../services/web/templates/partials/header.html")
 - After move to site/src/: include_str!("../../../../services/web/templates/partials/header.html")
 - Recommended: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../services/web/templates/partials/header.html"))

 Site Cargo.toml deps: systemprompt-web-shared, systemprompt, serde, serde_yaml, serde_json, async-trait, tracing, pulldown-cmark, comrak, axum, walkdir

 Verify: cargo check -p systemprompt-web-site && cargo check -p systemprompt-web-extension

 ---
 WAVE 5: Extract jobs crate + clean up root (18 files)

 Jobs cross-cut admin and content. This is last because it depends on both.

 ┌───────┬───────────────────────────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────────────────────────────────────────┐
 │ Agent │                             Task                              │                                                   Files                                                   │
 ├───────┼───────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 5-A   │ Create jobs/ crate scaffold + move content-only jobs (no      │ jobs/Cargo.toml, jobs/src/lib.rs, ingestion.rs, prerender.rs, publish.rs, sitemap.rs,                     │
 │       │ admin deps)                                                   │ content_analytics.rs, copy_assets.rs, llms_txt.rs, robots.rs, daily_summary/                              │
 ├───────┼───────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 5-B   │ Move admin-dependent jobs                                     │ compile_admin_templates.rs, bundle_admin_css.rs, bundle_admin_js.rs, recalculate_gamification.rs,         │
 │       │                                                               │ secret_migration.rs                                                                                       │
 ├───────┼───────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 5-C   │ Move marketplace jobs                                         │ marketplace_sync.rs, github_marketplace_sync.rs                                                           │
 ├───────┼───────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 5-D   │ Update workspace + root crate; wire pub use                   │ Cargo.toml, extensions/web/Cargo.toml, extensions/web/src/lib.rs                                          │
 │       │ systemprompt_web_jobs as jobs;                                │                                                                                                           │
 ├───────┼───────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 │ 5-E   │ Clean up root Cargo.toml — remove deps now only used by       │ Root Cargo.toml should only need: systemprompt, axum, sqlx, tokio, async-trait, tracing, serde_yaml,      │
 │       │ sub-crates; verify thin orchestrator                          │ inventory, + 5 sub-crate deps                                                                             │
 └───────┴───────────────────────────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────────────────────────────────────────┘

 Import rewrites in jobs:
 - use crate::admin::repositories → use systemprompt_web_admin::repositories
 - use crate::admin::gamification → use systemprompt_web_admin::gamification
 - use crate::admin::templates::helpers → use systemprompt_web_admin::templates::helpers
 - use crate::config:: → use systemprompt_web_shared::config::
 - use crate::services:: → use systemprompt_web_content::services::
 - use crate::models:: → use systemprompt_web_shared::models::

 Jobs Cargo.toml deps: systemprompt-web-shared, systemprompt-web-content, systemprompt-web-admin, systemprompt, sqlx, tokio, async-trait, tracing, serde_json, walkdir, anyhow, reqwest,
 handlebars

 Verify: cargo check -p systemprompt-web-jobs && cargo check --workspace

 ---
 WAVE 6: Final verification + cleanup

 ┌───────┬────────────────────────────────────────────────────────────────────────────┐
 │ Agent │                                    Task                                    │
 ├───────┼────────────────────────────────────────────────────────────────────────────┤
 │ 6-A   │ Audit root Cargo.toml — verify minimal dependency set, remove unused deps  │
 ├───────┼────────────────────────────────────────────────────────────────────────────┤
 │ 6-B   │ Audit pub visibility across all 5 sub-crates — minimize public API surface │
 ├───────┼────────────────────────────────────────────────────────────────────────────┤
 │ 6-C   │ Verify all include_str! paths compile and point to correct files           │
 ├───────┼────────────────────────────────────────────────────────────────────────────┤
 │ 6-D   │ Run cargo test --workspace — fix any test import breakage                  │
 ├───────┼────────────────────────────────────────────────────────────────────────────┤
 │ 6-E   │ Run cargo clippy --workspace — fix any new warnings                        │
 └───────┴────────────────────────────────────────────────────────────────────────────┘

 ---
 Final Root Crate (extensions/web/src/)

 After all waves, lib.rs should contain only:

 mod extension;
 mod extension_impl;
 mod config_loader;
 mod schemas;

 // Re-exports for API stability
 pub use systemprompt_web_shared as shared;
 pub use systemprompt_web_shared::{models, error, config, config_errors, utils};
 pub use systemprompt_web_admin as admin;
 pub use systemprompt_web_content::{repository, services, api};
 pub use systemprompt_web_site::{blog, docs, homepage, features, navigation, partials, extenders};
 pub use systemprompt_web_jobs as jobs;

 // Existing public re-exports (unchanged)
 pub use blog::{BlogListPageDataProvider, BlogPostPageDataProvider};
 pub use config::{BlogConfigRaw, BlogConfigValidated, ...};
 // ... etc

 Risk Mitigations

 ┌─────────────────────────────────────┬──────────────────────────────────────────────────────────────────────────────────────────────┐
 │                Risk                 │                                          Mitigation                                          │
 ├─────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ SecretCryptoError circular dep      │ Change MarketplaceError::Crypto(#[from] SecretCryptoError) to Crypto(String) in Wave 1       │
 ├─────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ include_str! path breakage          │ Use concat!(env!("CARGO_MANIFEST_DIR"), ...) pattern in Wave 4                               │
 ├─────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Missing re-exports break downstream │ Each wave verifies cargo check for both sub-crate and root                                   │
 ├─────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Admin crate too large for one wave  │ 5 agents split admin by directory: handlers, repositories, types, supporting modules, wiring │
 ├─────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Jobs cross-crate deps               │ Jobs extracted last (Wave 5) after both admin and content exist                              │
 └─────────────────────────────────────┴──────────────────────────────────────────────────────────────────────────────────────────────┘

 Verification (end-to-end)

 cargo check --workspace          # All crates compile
 cargo clippy --workspace         # No warnings
 cargo test --workspace           # All tests pass
 just build                       # Full build succeeds
 just start                       # Services start correctly
 systemprompt --help              # CLI works
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌

 Claude has written up a plan and is ready to execute. Would you like to proceed?

 ❯ 1. Yes, clear context (6% used) and bypass permissions
   2. Yes, and bypass permissions
   3. Yes, manually approve edits
   4. No, refine with Ultraplan on Claude Code on the web
   5. Tell Claude what to change
      shift+tab to approve with this feedback

 ctrl-g to edit in VS Code · ~/.claude/plans/glistening-baking-wombat.md